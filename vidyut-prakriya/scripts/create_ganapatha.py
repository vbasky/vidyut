"""Generates ganapatha.rs based on data from ashtadhyayi.com.

Usage:

    uv run create_ganapatha.py > ../src/ganapatha.rs
"""

import json
import urllib.request
from vidyut.lipi import transliterate, Scheme


REPLACE = {
    "pfTvAdiH": {
        "Uru": "uru",
    },
    "gahAdiH": {
        "antasTa": "antaHsTa",
    },
}

CODE_REPLACE = {
    "3.1.70": "2.1.70",
}

INSERT_AFTER = {"BOrikyAdiH": {"BOriki": ["vEpeya"]}}

EXPAND_COMMENT = {
    "sarvAdiH": {
        "<<pUrvaparAvaradakziRottarAparADarARivyavasTAyAmasaMjYAyAm>>": [
            "pUrva",
            "para",
            "avara",
            "dakziRa",
            "uttara",
            "apara",
            "aDara",
        ],
        "<<svamajYAtiDanAKyAyAm>>": ["sva"],
        "<<antaraM bahiryogopasaMvyAnayoH>>": ["antara"],
    },
    "kASyAdiH": {
        "<<ApadAdipUrvapadAtkAlAntAt>>": ["ApatkAla", "UrDvakAla", "tatkAla"],
    },
}

DELETE = {
    "kASyAdiH": {"Apad", "UrDva", "tat"},
    # TODO: check on this. marIci, if added here, seems to block mArIca.
    "bAhvAdiH": {"marIci"},
}


def load_ganapatha() -> dict:
    url = "https://raw.githubusercontent.com/ashtadhyayi-com/data/refs/heads/master/ganapath/data.txt"
    f = urllib.request.urlopen(url)
    return json.load(f)


def load_sutrapatha() -> dict:
    with open("../data/sutrapatha.tsv") as f:
        dp = f.read()

    map = {}
    for line in dp.splitlines():
        code, text = line.strip().split("\t")
        map[code] = text
    return map


def _to_const_name(slp_name: str) -> str:
    if slp_name == "bAhvAdiH":
        return "BAAHVADI"

    keys = "fFxXeEoOKGCJYwWqQRTDPBSz"
    values = [
        "R",
        "R",
        "L",
        "L",
        "E",
        "AI",
        "O",
        "AU",
        "KH",
        "GH",
        "CH",
        "JH",
        "N",
        "T",
        "TH",
        "D",
        "DH",
        "N",
        "TH",
        "DH",
        "PH",
        "BH",
        "SH",
        "SH",
    ]
    assert len(keys) == len(values)
    map = dict(zip(keys, values))

    buf = []
    for c in slp_name:
        buf.append(map.get(c, c))
    if slp_name.endswith("iH"):
        buf.pop()
    return "".join(buf).upper()


def main():
    sutrapatha = load_sutrapatha()
    ganapatha = load_ganapatha()

    print("""// Autogenerated by scripts/create_ganapatha.py

//! Implements rules from the *Gaṇapāṭha*.
//!
//! This module is largely auto-generated from the Ganapatha data on <https://ashtadhyayi.com>.
//! We have made a few minor corrections after the fact.
//!
//! For the source data, see <https://ashtadhyayi.com/ganapath>.

#![allow(unused)]

/// Models a *gaṇa-sūtra* from the *Gaṇapāṭha*.
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct GanapathaEntry {
    name: &'static str,
    number: u16,
    code: &'static str,
    items: &'static [&'static str],
    kind: GanaKind,
    varttika: Option<&'static str>
}

/// Models the *gaṇa* type.
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum GanaKind {
    /// A basic *gaṇa* whose members are all listed.
    Basic,
    /// A *gaṇa* whose members form an incomplete set. The full set can be known only by
    /// observing actual usage.
    Akrti,
}

impl GanapathaEntry {
    pub(crate) const fn basic(
        name: &'static str,
        number: u16,
        code: &'static str,
        items: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            number,
            code,
            items,
            kind: GanaKind::Basic,
            varttika: None,
        }
    }

    pub(crate) const fn akrti(
        name: &'static str,
        number: u16,
        code: &'static str,
        items: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            number,
            code,
            items,
            kind: GanaKind::Akrti,
            varttika: None,
        }
    }

    pub(crate) const fn with_varttika(mut self: GanapathaEntry, varttika: &'static str) -> Self {
        self.varttika = Some(varttika);
        self
    }

    /// The name of this *gaṇa* in SLP1 transliteration.
    pub fn name(&self) -> &str {
        self.name
    }

    /// The number of this *gaṇa* relative to other *gaṇa*s in the Ganapatha.
    pub fn number(&self) -> u16 {
        self.number
    }

    /// The string ID of the Ashtadhyayi rule that first uses this *gaṇa*.
    pub fn code(&self) -> &str {
        self.code
    }

    /// All items in the *gaṇa*.
    pub fn items(&self) -> &[&str] {
        self.items
    }

    /// The type of this *gaṇa*.
    pub fn kind(&self) -> GanaKind {
        self.kind
    }

    /// A *vārttika* associated with this *gaṇa*, if one exists.
    pub fn varttika(&self) -> Option<&str> {
        self.varttika
    }
}

""")

    seen = set()
    ordered_names = []
    for sutra in ganapatha["data"]:
        index = sutra["ind"]

        name = transliterate(sutra["name"], Scheme.Devanagari, Scheme.Slp1)
        if name == "ugavAdiH":
            name = "gavAdiH"
        elif name == "kattryAdiH":
            name = "katryAdiH"
        else:
            name = name.replace("Mk", "Nk")
            name = name.replace("Md", "nd")
            name = name.replace("MD", "nD")

        const_name = _to_const_name(name)
        if const_name in seen:
            # Disambiguate with the adhyAya number.
            const_name += "_" + code[0]
        seen.add(const_name)
        ordered_names.append(const_name)

        code = sutra["sutra"]
        code = CODE_REPLACE.get(code, code)

        words = transliterate(sutra["words"], Scheme.Devanagari, Scheme.Slp1)
        words = [x.strip() for x in words.split(".") if x.strip()]

        code_text = transliterate(sutrapatha[code], Scheme.Slp1, Scheme.Iso15919)
        varttika_text = transliterate(sutra["vartika"], Scheme.Devanagari, Scheme.Slp1)
        is_akrti = sutra["type"] == "A"

        name_iso = transliterate(name[:-1], Scheme.Slp1, Scheme.Iso15919)
        print(f"/// *{name_iso}-gaṇa* ({index}), first used in the following *sūtra*:")
        print("///")
        print(f"/// > {code} *{code_text}*")
        if varttika_text:
            print("/// The *sūtra* has the following *vārttika*:")
            print("///")
            print(f"/// > *{varttika_text}*")

        if is_akrti:
            print(f"pub(crate) const {const_name}: GanapathaEntry = GanapathaEntry::akrti(")
        else:
            print(f"pub(crate) const {const_name}: GanapathaEntry = GanapathaEntry::basic(")
        print(f'"{name}", {index}, "{code}", &[')
        for w in words:
            if w in DELETE.get(name, set()):
                continue

            w = REPLACE.get(name, {}).get(w, w)
            extras = INSERT_AFTER.get(name, {}).get(w, [])

            if w.startswith("<") or " " in w:
                print(f"    // {w}")
                expanded = EXPAND_COMMENT.get(name, {}).get(w, [])
                for e in expanded:
                    print(f'    "{e}",')
            else:
                print(f'    "{w}",')

            for e in extras:
                print(f'    "{e}",')

        if varttika_text:
            print(f']).with_varttika("{varttika_text}");\n')
        else:
            print("]);\n")

    print("""

/// Returns an ordered iterator over all *gaṇa-sūtra*s.
pub fn all_sutras() -> impl Iterator<Item = &'static GanapathaEntry> {
    const SUTRAS: &[GanapathaEntry] = &[""")

    for name in ordered_names:
        print(f"        {name},")

    print("""];
    SUTRAS.iter()
}""")


if __name__ == "__main__":
    main()
