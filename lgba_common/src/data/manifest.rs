use crate::hashes::hashed;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    hash::Hash,
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ManifestRoot {
    pub name: String,
    #[serde(default)]
    pub spec: Option<String>,
    #[serde(default)]
    pub partitions: BTreeMap<String, String>,
    #[serde(default)]
    pub filters: Vec<String>,
}
impl ManifestRoot {
    pub fn all_partitions(&self) -> Result<BTreeMap<String, String>> {
        let mut partitions = self.partitions.clone();
        if self.spec.is_some() {
            if partitions.contains_key("data") {
                bail!("Duplicate data partition (the `spec` field creates a `data` partition.)");
            }
            partitions.insert(String::from("data"), self.spec.clone().unwrap());
        }
        if partitions.is_empty() {
            bail!("No partitions found!");
        }
        Ok(partitions)
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct FilesystemManifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub root: Vec<ManifestRoot>,
}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParsedSpecInclusion {
    Str,
    U16,
    U16Hex,
    U32,
    U32Hex,
    Wildcard,
    WildcardDir,
}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParsedSpecShape {
    Str,
    U16,
    U16U16,
    U32,
}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParsedSpec {
    pub segments: Vec<String>,
    pub spec: Vec<ParsedSpecInclusion>,
}
impl ParsedSpec {
    fn parse(raw_text: &str) -> Result<Option<ParsedSpec>> {
        if raw_text == "*" {
            Ok(None)
        } else {
            let text = raw_text.as_bytes();

            let mut segments = Vec::new();
            let mut spec = Vec::new();

            let mut start_idx = 0;
            let mut idx = 0;
            loop {
                match text.get(idx) {
                    Some(&b'*') => {
                        segments.push(raw_text[start_idx..idx].to_string());

                        let mut star_count = 0;
                        while text.get(idx) == Some(&b'*') {
                            star_count += 1;
                            idx += 1;
                        }

                        if star_count == 1 {
                            spec.push(ParsedSpecInclusion::Wildcard);
                        } else {
                            spec.push(ParsedSpecInclusion::WildcardDir);
                        }

                        start_idx = idx;
                    }
                    Some(&b'{') => {
                        segments.push(raw_text[start_idx..idx].to_string());

                        let seg_start = idx + 1;
                        while idx < text.len() && text.get(idx) != Some(&b'}') {
                            idx += 1;
                        }
                        let seg_end = idx;
                        ensure!(
                            seg_end < text.len() && text.get(idx) == Some(&b'}'),
                            "Bracket does not match in file spec."
                        );
                        idx += 1;

                        match &raw_text[seg_start..seg_end] {
                            "str" => spec.push(ParsedSpecInclusion::Str),
                            "u16" => spec.push(ParsedSpecInclusion::U16),
                            "u16x" => spec.push(ParsedSpecInclusion::U16Hex),
                            "u32" => spec.push(ParsedSpecInclusion::U32),
                            "u32x" => spec.push(ParsedSpecInclusion::U32Hex),
                            seg => bail!("Invalid segment type: {{{seg}}}"),
                        }

                        start_idx = idx;
                    }
                    Some(_) => idx += 1,
                    None => break,
                }
            }
            ensure!(!spec.is_empty(), "File spec must contain at least one variable segment.");
            segments.push(raw_text[start_idx..].to_string());

            let parsed = ParsedSpec { segments, spec };
            parsed.check()?;
            parsed.shape()?;
            Ok(Some(parsed))
        }
    }

    fn check(&self) -> Result<()> {
        ensure!(
            self.segments.len() == self.spec.len() + 1 && self.spec.len() > 0,
            "no variable segments found"
        );
        for i in 1..self.segments.len() - 1 {
            ensure!(!self.segments[i].is_empty(), "interior segments should not be empty");
        }
        Ok(())
    }

    pub fn shape(&self) -> Result<ParsedSpecShape> {
        self.check()?;

        #[derive(Debug)]
        enum Spec {
            Str,
            U16,
            U32,
        }
        let inclusions: Vec<_> = self
            .spec
            .iter()
            .filter_map(|x| match x {
                ParsedSpecInclusion::Str => Some(Spec::Str),
                ParsedSpecInclusion::U16 => Some(Spec::U16),
                ParsedSpecInclusion::U16Hex => Some(Spec::U16),
                ParsedSpecInclusion::U32 => Some(Spec::U32),
                ParsedSpecInclusion::U32Hex => Some(Spec::U32),
                ParsedSpecInclusion::Wildcard => None,
                ParsedSpecInclusion::WildcardDir => None,
            })
            .collect();
        match inclusions.as_slice() {
            &[Spec::Str] => Ok(ParsedSpecShape::Str),
            &[Spec::U16] => Ok(ParsedSpecShape::U16),
            &[Spec::U16, Spec::U16] => Ok(ParsedSpecShape::U16U16),
            &[Spec::U32] => Ok(ParsedSpecShape::U32),
            _ => bail!("Invalid spec shape: {:?}", inclusions),
        }
    }

    #[cfg(feature = "data_build")]
    pub fn glob(&self) -> Result<String> {
        self.check()?;

        let mut accum = String::new();
        for (prefix, spec) in self.segments.iter().zip(self.spec.iter()) {
            accum.push_str(prefix);
            match spec {
                ParsedSpecInclusion::Str => accum.push_str("*"),
                ParsedSpecInclusion::U16 => accum.push_str("*"),
                ParsedSpecInclusion::U16Hex => accum.push_str("*"),
                ParsedSpecInclusion::U32 => accum.push_str("*"),
                ParsedSpecInclusion::U32Hex => accum.push_str("*"),
                ParsedSpecInclusion::Wildcard => accum.push_str("*"),
                ParsedSpecInclusion::WildcardDir => accum.push_str("**"),
            }
        }
        accum.push_str(self.segments.last().unwrap());
        Ok(accum)
    }

    #[cfg(feature = "data_build")]
    pub fn regex(&self) -> Result<String> {
        self.check()?;

        let mut accum = String::new();
        for (prefix, spec) in self.segments.iter().zip(self.spec.iter()) {
            accum.push_str(prefix);
            match spec {
                ParsedSpecInclusion::Str => accum.push_str("([^/]+)"),
                ParsedSpecInclusion::U16 => accum.push_str("([0-9]+)"),
                ParsedSpecInclusion::U16Hex => accum.push_str("([0-9a-fA-F]+)"),
                ParsedSpecInclusion::U32 => accum.push_str("([0-9]+)"),
                ParsedSpecInclusion::U32Hex => accum.push_str("([0-9a-fA-F]+)"),
                ParsedSpecInclusion::Wildcard => accum.push_str("[^/]*"),
                ParsedSpecInclusion::WildcardDir => accum.push_str(".*"),
            }
        }
        accum.push_str(self.segments.last().unwrap());
        Ok(accum)
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ParsedRoot {
    pub name: String,
    pub partitions: BTreeMap<String, Option<ParsedSpec>>,
    pub filters: Vec<String>,
}
impl ParsedRoot {
    fn parse(data: ManifestRoot) -> Result<ParsedRoot> {
        let mut partitions = BTreeMap::new();
        for (name, partition) in data.partitions {
            partitions.insert(name, ParsedSpec::parse(&partition)?);
        }
        Ok(ParsedRoot { name: data.name, partitions, filters: data.filters })
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ParsedManifest {
    pub name: Option<String>,
    pub roots: BTreeMap<String, ParsedRoot>,
}
impl ParsedManifest {
    pub fn parse(data: &str) -> Result<ParsedManifest> {
        let raw_manifest = toml::from_str::<FilesystemManifest>(data)?;
        Ok(ParsedManifest::parse_raw(raw_manifest)?)
    }

    pub fn parse_raw(data: FilesystemManifest) -> Result<ParsedManifest> {
        let mut roots = BTreeMap::new();
        for root in data.root {
            ensure!(!roots.contains_key(&root.name), "Duplicate root: {}", root.name);
            roots.insert(root.name.clone(), ParsedRoot::parse(root)?);
        }
        Ok(ParsedManifest { name: data.name, roots })
    }

    pub fn hash(&self) -> [u8; 12] {
        let mut sub_hash = [0; 12];
        sub_hash.copy_from_slice(&hashed(self, 0)[..12]);
        sub_hash
    }
}

#[cfg(test)]
mod test {
    use crate::data::{ParsedSpec, ParsedSpecInclusion};
    use std::vec;

    #[test]
    fn test_spec_parse() {
        assert_eq!(
            ParsedSpec::parse("abc{u16}def").unwrap(),
            Some(ParsedSpec {
                segments: vec!["abc".into(), "def".into()],
                spec: vec![ParsedSpecInclusion::U16],
            })
        );
        assert_eq!(
            ParsedSpec::parse("abc{u16}a{u16}def").unwrap(),
            Some(ParsedSpec {
                segments: vec!["abc".into(), "a".into(), "def".into()],
                spec: vec![ParsedSpecInclusion::U16, ParsedSpecInclusion::U16],
            })
        );
        assert_eq!(
            ParsedSpec::parse("abc{u16}b******c{u16}def").unwrap(),
            Some(ParsedSpec {
                segments: vec!["abc".into(), "b".into(), "c".into(), "def".into()],
                spec: vec![
                    ParsedSpecInclusion::U16,
                    ParsedSpecInclusion::WildcardDir,
                    ParsedSpecInclusion::U16
                ],
            })
        );
        assert!(ParsedSpec::parse("asf{asf").is_err());
        assert!(ParsedSpec::parse("abcdefg").is_err());
        assert!(ParsedSpec::parse("abcdefg{not_valid}").is_err());
        assert!(ParsedSpec::parse("abcdefg{u16}a{u16}a{u16}").is_err());
        assert!(ParsedSpec::parse("abcdefg{u16}{u16}abc").is_err());
    }
}
