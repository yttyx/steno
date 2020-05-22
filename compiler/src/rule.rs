use std::collections::HashMap;

use regex::Regex;

pub type Dict = HashMap<String, String>;

lazy_static! {
    static ref ENTRY_SUFFIX: Regex = Regex::new(r"^\{\^([[:alpha:]]*)\}$").unwrap();
}

#[derive(Debug, Deserialize)]
struct Rule {
    word: String,
    suffix: String,
    repl: String,
}

#[derive(Deserialize)]
pub struct Rules {
    regex: Vec<Rule>,
    simple: Vec<Rule>,
    word_list: HashMap<String, u8>,
}

pub fn apply_rules(rules: &Rules, source: &Dict) -> Dict {
    let suffixes: Dict = source
        .iter()
        .filter_map(|(stroke, entry)| {
            if ENTRY_SUFFIX.is_match(entry) {
                Some((
                    stroke.clone(),
                    ENTRY_SUFFIX.replace(&entry[..], "${1}").into(),
                ))
            } else {
                None
            }
        })
        .collect();
    let mut cached_res = HashMap::new();
    let mut output: Dict = HashMap::new();
    for (stroke, entry) in source.into_iter() {
        output.insert(stroke.clone(), entry.clone());
        for rule in rules.simple.iter() {
            if &rule.word == entry {
                let new_stroke = suffixes.iter().find_map(|(k, v)| {
                    if v == &rule.suffix {
                        Some(k.clone())
                    } else {
                        None
                    }
                });
                if let Some(new_stroke) = new_stroke {
                    output.insert(format!("{}/{}", stroke, new_stroke), rule.repl.clone());
                } else {
                    // println!("{:?}", rule);
                }
            }
        }
        for rule in rules.regex.iter() {
            if !cached_res.contains_key(&rule.word) {
                cached_res.insert(rule.word.clone(), Regex::new(&rule.word).unwrap());
            }
            let word_re = cached_res.get(&rule.word).unwrap();
            if word_re.is_match(&entry) {
                let pat = format!(
                    "{}\u{ffff}{}",
                    &rule.word[..(rule.word.len() - 1)],
                    rule.suffix
                );
                if !cached_res.contains_key(&pat) {
                    cached_res.insert(pat.clone(), Regex::new(&pat).unwrap());
                }

                for (new_stroke, suffix) in suffixes.iter() {
                    let simple_add = format!("{}\u{ffff}{}", entry, suffix);
                    let pat_re = cached_res.get(&pat).unwrap();
                    if pat_re.is_match(&simple_add) {
                        let word = pat_re.replace(&simple_add[..], &rule.repl[..]);
                        if rules.word_list.contains_key(word.as_ref()) {
                            output.insert(format!("{}/{}", stroke, new_stroke), word.into());
                        }
                    }
                }
            }
        }
    }
    output
}