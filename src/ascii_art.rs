/// ASCII art digit representations for the timer display
/// Each digit is represented as a 5-line array of strings

const DIGIT_0: [&str; 5] = [
    " ███ ",
    "█   █",
    "█   █",
    "█   █",
    " ███ ",
];

const DIGIT_1: [&str; 5] = [
    "  █  ",
    " ██  ",
    "  █  ",
    "  █  ",
    " ███ ",
];

const DIGIT_2: [&str; 5] = [
    " ███ ",
    "█   █",
    "  ██ ",
    " █   ",
    "█████",
];

const DIGIT_3: [&str; 5] = [
    " ███ ",
    "█   █",
    "  ██ ",
    "█   █",
    " ███ ",
];

const DIGIT_4: [&str; 5] = [
    "█   █",
    "█   █",
    "█████",
    "    █",
    "    █",
];

const DIGIT_5: [&str; 5] = [
    "█████",
    "█    ",
    "████ ",
    "    █",
    "████ ",
];

const DIGIT_6: [&str; 5] = [
    " ███ ",
    "█    ",
    "████ ",
    "█   █",
    " ███ ",
];

const DIGIT_7: [&str; 5] = [
    "█████",
    "    █",
    "   █ ",
    "  █  ",
    "  █  ",
];

const DIGIT_8: [&str; 5] = [
    " ███ ",
    "█   █",
    " ███ ",
    "█   █",
    " ███ ",
];

const DIGIT_9: [&str; 5] = [
    " ███ ",
    "█   █",
    " ████",
    "    █",
    " ███ ",
];

const COLON: [&str; 5] = [
    "     ",
    "  █  ",
    "     ",
    "  █  ",
    "     ",
];

const SPACE: [&str; 5] = [
    "     ",
    "     ",
    "     ",
    "     ",
    "     ",
];

/// Gets the ASCII art representation for a single character
fn get_char_art(c: char) -> [&'static str; 5] {
    match c {
        '0' => DIGIT_0,
        '1' => DIGIT_1,
        '2' => DIGIT_2,
        '3' => DIGIT_3,
        '4' => DIGIT_4,
        '5' => DIGIT_5,
        '6' => DIGIT_6,
        '7' => DIGIT_7,
        '8' => DIGIT_8,
        '9' => DIGIT_9,
        ':' => COLON,
        _ => SPACE,
    }
}

/// Converts a time string (e.g., "25:00") into ASCII art lines
/// Returns a Vec of 5 strings, one for each line of the ASCII art
pub fn time_to_ascii_art(time_str: &str) -> Vec<String> {
    let mut lines: Vec<String> = vec![String::new(); 5];
    
    for c in time_str.chars() {
        let art = get_char_art(c);
        for (i, line) in art.iter().enumerate() {
            lines[i].push_str(line);
            lines[i].push(' '); // Add spacing between characters
        }
    }
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_to_ascii_art() {
        let result = time_to_ascii_art("12:34");
        assert_eq!(result.len(), 5);
        // Each line should have content
        for line in &result {
            assert!(!line.is_empty());
        }
    }
}
