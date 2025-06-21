use vt100::Parser;

pub struct ScreenManager {
    parser: Parser,
    current_snapshot: String,
    previous_snapshot: String,
}

impl ScreenManager {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: Parser::new(rows, cols, 0),
            current_snapshot: String::new(),
            previous_snapshot: String::new(),
        }
    }

    pub fn process(&mut self, data: &[u8]) {
        self.parser.process(data);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser = Parser::new(rows, cols, 0);
    }

    pub fn take_snapshot(&mut self) -> (String, String) {
        self.previous_snapshot = std::mem::take(&mut self.current_snapshot);
        self.current_snapshot = self.screen_to_string();
        (
            self.previous_snapshot.clone(),
            self.current_snapshot.clone(),
        )
    }

    pub fn get_snapshots(&self) -> (&str, &str) {
        (&self.previous_snapshot, &self.current_snapshot)
    }

    fn screen_to_string(&self) -> String {
        let screen = self.parser.screen();
        let mut lines = Vec::new();

        for row in 0..screen.size().0 {
            let mut line = String::new();
            for col in 0..screen.size().1 {
                let cell = screen.cell(row, col).unwrap();
                line.push_str(&cell.contents());
            }
            let trimmed = line.trim_end();
            lines.push(trimmed.to_string());
        }

        while lines.len() > 0 && lines.last().unwrap().is_empty() {
            lines.pop();
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_snapshots() {
        let mut screen = ScreenManager::new(24, 80);

        screen.process(b"Hello, World!");
        let (prev, curr) = screen.take_snapshot();
        assert_eq!(prev, "");
        assert!(curr.contains("Hello, World!"));

        screen.process(b"\r\nSecond line");
        let (prev, curr) = screen.take_snapshot();
        assert!(prev.contains("Hello, World!"));
        assert!(curr.contains("Second line"));
    }
}
