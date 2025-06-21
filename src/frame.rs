use tokio::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 16;
const MAX_BYTES_WITHOUT_NEWLINE: usize = 4096;

pub struct FrameDetector {
    last_data_time: Instant,
    bytes_since_newline: usize,
    debounce_duration: Duration,
}

impl FrameDetector {
    pub fn new() -> Self {
        Self {
            last_data_time: Instant::now(),
            bytes_since_newline: 0,
            debounce_duration: Duration::from_millis(DEBOUNCE_MS),
        }
    }
    
    pub fn on_data(&mut self, data: &[u8]) {
        self.last_data_time = Instant::now();
        
        for &byte in data {
            if byte == b'\n' {
                self.bytes_since_newline = 0;
            } else {
                self.bytes_since_newline += 1;
            }
        }
    }
    
    pub fn should_capture_frame(&self) -> bool {
        let elapsed = Instant::now().duration_since(self.last_data_time);
        
        elapsed >= self.debounce_duration || 
        self.bytes_since_newline > MAX_BYTES_WITHOUT_NEWLINE
    }
    
    pub fn reset(&mut self) {
        self.bytes_since_newline = 0;
    }
}

impl Default for FrameDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_debounce_detection() {
        let mut detector = FrameDetector::new();
        
        detector.on_data(b"Hello");
        assert!(!detector.should_capture_frame());
        
        sleep(Duration::from_millis(20)).await;
        assert!(detector.should_capture_frame());
    }
    
    #[test]
    fn test_large_data_detection() {
        let mut detector = FrameDetector::new();
        
        let large_data = vec![b'a'; MAX_BYTES_WITHOUT_NEWLINE + 100];
        detector.on_data(&large_data);
        
        assert!(detector.should_capture_frame());
    }
}