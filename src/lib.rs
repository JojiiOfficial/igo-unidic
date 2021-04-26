use std::{
    convert::{TryFrom, TryInto},
    path::Path,
};

use igo::Morpheme as IgoMorpheme;
use igo::Tagger;

pub struct Parser {
    parser: Tagger,
}

impl Parser {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let tagger = Tagger::new(Path::new(path))?;
        Ok(Parser { parser: tagger })
    }

    pub fn parse<'text, 'dict>(&'dict self, text: &'text str) -> Vec<Morpheme<'dict, 'text>> {
        self.parser
            .parse(text)
            .into_iter()
            .map(Morpheme::from)
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Morpheme<'dict, 'input> {
    pub surface: &'input str,
    pub basic: &'dict str,
    pub word_class: WordClass<'dict>,
    pub conjungation: Conjungation,
    pub origin: Option<Origin>,
    pub reading: &'dict str,
    pub lexeme: &'dict str,
    pub start: usize,
}

impl<'dict, 'input> From<IgoMorpheme<'dict, 'input>> for Morpheme<'dict, 'input> {
    fn from(igo_morph: IgoMorpheme<'dict, 'input>) -> Morpheme<'dict, 'input> {
        println!("{:#?}", igo_morph);
        let features: &Vec<_> = &igo_morph.feature.split(',').collect();

        let word_class: WordClass = features.try_into().unwrap();

        let conjungation_form: ConjungationForm = features.try_into().unwrap();
        let conjungation_kind: ConjungationKind = features.try_into().unwrap();

        let conjungation = Conjungation {
            kind: conjungation_kind,
            form: conjungation_form,
        };

        let origin = if features.len() > 12 {
            Origin::from_string(features[12])
        } else {
            None
        };

        let basic = str_or_empty(features, 6);
        let lexeme = str_or_empty(features, 10);
        let reading = str_or_empty(features, 9);

        Morpheme {
            start: igo_morph.start,
            surface: igo_morph.surface,
            basic,
            lexeme,
            reading,
            word_class,
            origin,
            conjungation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Conjungation {
    pub kind: ConjungationKind,
    pub form: ConjungationForm,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ConjungationKind {
    None,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ConjungationForm {
    None,
    Plain,
    Imperative,
    Negative,
    Attributive,
    Continuous,
    Conditional,
    Stem,
    Realis,
    Kugohou,
}

impl<'a> TryFrom<&Vec<&'a str>> for ConjungationKind {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        Ok(match value[4] {
            _ => Self::None,
            //_ => return Err(format!("conjungation kind not found {}", value[4])),
        })
    }
}

impl<'a> TryFrom<&Vec<&'a str>> for ConjungationForm {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        let t = split_type(value[5]).0;
        Ok(match t {
            "*" => ConjungationForm::None,
            "終止形" => ConjungationForm::Plain,
            "命令形" => ConjungationForm::Imperative,
            "未然形" => ConjungationForm::Negative,
            "連体形" => ConjungationForm::Attributive,
            "連用形" => ConjungationForm::Continuous,
            "仮定形" => ConjungationForm::Conditional,
            "語幹" => ConjungationForm::Stem,
            "已然形" => ConjungationForm::Realis,
            "意志推量形" => ConjungationForm::None, // wtf is this?
            "ク語法" => ConjungationForm::Kugohou,
            _ => return Err(format!("conjungation form not found {}", t)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum WordClass<'a> {
    Particle(ParticleType),
    Verb(VerbType<'a>),
    Adjective(AdjectiveType),
    Adverb,
    Noun(NounType),
    Pronoun,
    Interjection,
    Symbol,
    Conjungtion,
    Suffix,
    Prefix,
    PreNoun,
}

impl<'a> TryFrom<&Vec<&'a str>> for WordClass<'a> {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        Ok(match value[0] {
            "助詞" => WordClass::Particle(ParticleType::try_from(value)?),
            "形容詞" | "形状詞" => WordClass::Adjective(AdjectiveType::try_from(value)?),
            "助動詞" | "動詞" => WordClass::Verb(VerbType::try_from(value)?),
            "代名詞" => WordClass::Pronoun,
            "感動詞" => WordClass::Interjection,
            "補助記号" | "記号" => WordClass::Symbol,
            "接続詞" => WordClass::Conjungtion,
            "接尾辞" => WordClass::Suffix,
            "接頭辞" => WordClass::Prefix,
            "副詞" => WordClass::Adverb,
            "名詞" => WordClass::Noun(NounType::try_from(value)?),
            "連体詞" => WordClass::PreNoun,
            _ => return Err(format!("wc not found {}", value[0])),
        })
    }
}

//
// ------ Noun
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum NounType {
    Common,
    Proper,
    Numeral,
}

impl TryFrom<&Vec<&str>> for NounType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[1] {
            "普通名詞" => Self::Common,
            "固有名詞" => Self::Proper,
            "数詞" => Self::Numeral,
            _ => return Err(format!("Nountype not found {}", value[1])),
        })
    }
}

//
// ------ Particle
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ParticleType {
    Connecting,
    SentenceEnding,
    CaseMaking,
    Conjungtion,
    Adverbial,
    Nominalizing,
}

impl TryFrom<&Vec<&str>> for ParticleType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[1] {
            "係助詞" => Self::Connecting,
            "終助詞" => Self::SentenceEnding,
            "格助詞" => Self::CaseMaking,
            "接続助詞" => Self::Conjungtion,
            "副助詞" => Self::Adverbial,
            "準体助詞" => Self::Nominalizing,
            _ => return Err(format!("particle not found {}", value[1])),
        })
    }
}

//
// ------ Adjective
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum AdjectiveType {
    I,
    Na,
}

impl TryFrom<&Vec<&str>> for AdjectiveType {
    type Error = String;
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        Ok(match value[0] {
            "形容詞" => Self::I,
            "形状詞" => Self::Na,
            _ => return Err(format!("adjective not found {}", value[0])),
        })
    }
}

//
// ------ Verb
//
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum VerbType<'a> {
    Auxilary(&'a str),
    Godan(SyllableRow),
    Ichidan(SyllableRow),
    IchidanEruConjungation,
    Suru,
    Kuru,
    IrregRu,
    IrregNu,
}

impl<'a> TryFrom<&Vec<&'a str>> for VerbType<'a> {
    type Error = String;
    fn try_from(value: &Vec<&'a str>) -> Result<Self, Self::Error> {
        let verb_type = value[4];

        Ok(match value[0] {
            "助動詞" => Self::Auxilary(split_type(verb_type).1),
            "動詞" => Self::parse_general(verb_type)?,
            _ => return Err(format!("Verb not found {}", value[0])),
        })
    }
}

impl<'a> VerbType<'a> {
    fn parse_general(verb_type: &'a str) -> Result<Self, String> {
        let verb_type = split_type(verb_type);
        Ok(match verb_type.0 {
            "五段" | "文語下二段" | "文語四段" | "文語上二段" | "上二段" => {
                VerbType::Godan(SyllableRow::try_from(verb_type.1)?)
            }
            "一段" | "下一段" | "上一段" => {
                VerbType::Ichidan(SyllableRow::try_from(verb_type.1)?)
            }
            "サ行変格" | "文語サ行変格" => VerbType::Suru,
            "カ行変格" => VerbType::Kuru,
            "ラ行変格" | "文語ラ行変格" => VerbType::IrregRu,
            "ナ行変格" | "文語ナ行変格" => VerbType::IrregNu,
            _ => return Err(format!("Verbtype {} not found", verb_type.0)),
        })
    }
}

//
// ------ Other
//

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum SyllableRow {
    G,
    K,
    M,
    A,
    R,
    S,
    Z,
    T,
    D,
    B,
    H,
    P,
    N,
    Wa,
    Y,
}

impl<'a> TryFrom<&'a str> for SyllableRow {
    type Error = String;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "ガ行" => SyllableRow::G,
            "カ行" => SyllableRow::K,
            "マ行" => SyllableRow::M,
            "ア行" => SyllableRow::A,
            "ラ行" => SyllableRow::R,
            "サ行" => SyllableRow::S,
            "ザ行" => SyllableRow::Z,
            "タ行" => SyllableRow::T,
            "ダ行" => SyllableRow::D,
            "ハ行" => SyllableRow::H,
            "バ行" => SyllableRow::B,
            "パ行" => SyllableRow::P,
            "ナ行" => SyllableRow::N,
            "ワア行" => SyllableRow::Wa,
            "ヤ行" => SyllableRow::Y,
            _ => return Err(format!("Syllable ending not found {}", value)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Origin {
    China,
    Japan,
}

impl Origin {
    fn from_string(s: &str) -> Option<Self> {
        match s {
            "漢" => Some(Self::China),
            "和" => Some(Self::Japan),
            _ => None,
        }
    }
}

//
// Helper
//

/// Splits a type definition and gets both sides
fn split_type<'a>(inp: &'a str) -> (&'a str, &'a str) {
    let mut s = inp.split("-");
    (s.next().unwrap_or(""), s.next().unwrap_or(""))
}

fn str_or_empty<'a>(vec: &Vec<&'a str>, pos: usize) -> &'a str {
    if vec.len() > pos {
        vec[pos]
    } else {
        ""
    }
}
