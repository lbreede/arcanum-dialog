use std::convert::TryFrom;
#[derive(Debug, Clone)]
pub struct DialogLine {
    pub number: usize,
    pub text: String,
    pub female_text: Option<String>,
    pub intelligence: Option<i32>,
    pub test: Option<String>,
    pub response: Option<usize>,
    pub result: Option<String>,
    pub choices: Vec<usize>,
}

impl TryFrom<String> for DialogLine {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<String> = value
            .match_indices('{')
            .zip(value.match_indices('}'))
            .map(|((start, _), (end, _))| &value[start + 1..end])
            .take(7)
            .map(str::trim)
            .map(String::from)
            .collect();
        let number = parts
            .first()
            .ok_or_else(|| "`number` field is missing".to_string())?
            .parse::<usize>()
            .map_err(|_| "`number` field must be a valid usize".to_string())?;
        let text = parts
            .get(1)
            .ok_or_else(|| "`text` field is missing".to_string())?
            .clone();
        let female_text = parts.get(2).filter(|s| !s.is_empty()).cloned();
        let intelligence = parts.get(3).and_then(|s| s.parse::<i32>().ok());
        let test = parts.get(4).filter(|s| !s.is_empty()).cloned();
        let response = parts.get(5).and_then(|s| s.parse::<usize>().ok());
        let result = parts.get(6).filter(|s| !s.is_empty()).cloned();

        Ok(Self {
            number,
            text,
            female_text,
            intelligence,
            test,
            response,
            result,
            choices: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_string() {
        match DialogLine::try_from(
            "{1}{How can I help you?}{How can I help you?}{}{}{}{}".to_string(),
        ) {
            Ok(dialog) => {
                assert_eq!(dialog.number, 1);
                assert_eq!(dialog.text, "How can I help you?".to_string());
                assert_eq!(dialog.female_text, Some("How can I help you?".to_string()));
                assert_eq!(dialog.intelligence, None);
                assert_eq!(dialog.test, None);
                assert_eq!(dialog.response, None);
                assert_eq!(dialog.result, None);
                assert!(dialog.choices.is_empty());
            }
            Err(e) => panic!("Expected successful conversion, but got error: {}", e),
        }

        match DialogLine::try_from("{2}{B:}{}{5}{}{8}{}Barter".to_string()) {
            Ok(dialog) => {
                assert_eq!(dialog.number, 2);
                assert_eq!(dialog.text, "B:".to_string());
                assert_eq!(dialog.female_text, None);
                assert_eq!(dialog.intelligence, Some(5));
                assert_eq!(dialog.test, None);
                assert_eq!(dialog.response, Some(8));
                assert_eq!(dialog.result, None);
                assert!(dialog.choices.is_empty());
            }
            Err(e) => panic!("Expected successful conversion, but got error: {}", e),
        }

        match DialogLine::try_from("{6}{Let’s Go.}{}{5}{wa}{0}{uw}Stop Waiting".to_string()) {
            Ok(dialog) => {
                assert_eq!(dialog.number, 6);
                assert_eq!(dialog.text, "Let’s Go.".to_string());
                assert_eq!(dialog.female_text, None);
                assert_eq!(dialog.intelligence, Some(5));
                assert_eq!(dialog.test, Some("wa".to_string()));
                assert_eq!(dialog.response, Some(0));
                assert_eq!(dialog.result, Some("uw".to_string()));
                assert!(dialog.choices.is_empty());
            }
            Err(e) => panic!("Expected successful conversion, but got error: {}", e),
        }
    }
}
