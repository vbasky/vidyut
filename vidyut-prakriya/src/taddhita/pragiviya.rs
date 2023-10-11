/*!
Implements the taddhita rules in the "prAg ivAt kaH" section of pada 5.3, as well as some rules
immediately before it.

(5.3.27 - 5.3.95)
*/
use crate::args::Taddhita::*;
use crate::args::TaddhitaArtha::*;
use crate::taddhita::utils::TaddhitaPrakriya;
use crate::tag::Tag;

fn try_base_cases(tp: &mut TaddhitaPrakriya, _rule: &'static str) {
    let prati = tp.prati();
    if prati.is_sarvanama() {
        // TODO: this is incorrect.
        tp.try_add("5.3.71", akac);
    } else {
        tp.try_add("5.3.70", ka);
    }
}

pub fn run(tp: &mut TaddhitaPrakriya) {
    tp.with_context(DigDeshaKala, |tp| {
        let prati = tp.prati();
        if prati.has_text_in(&["dakziRa", "uttara"]) {
            tp.try_add("5.3.28", atasuc);
        } else if prati.has_text_in(&["para", "avara"]) {
            tp.optional_try_add("5.3.29", atasuc);
        }
        if !tp.has_taddhita {
            tp.try_add("5.3.27", astAti);
        }
    });

    let prati = tp.prati();
    if prati.has_tag(Tag::Sankhya) {
        tp.try_add("5.3.42", DA);
    }

    let code = "5.3.55";
    tp.try_add(code, tamap);
    tp.try_add(code, izWan);

    let code = "5.3.57";
    tp.try_add(code, tarap);
    tp.try_add(code, Iyasun);

    // vaiyAkaraRarUpa
    tp.try_add("5.3.66", rUpap);

    // pawukalpa
    let code = "5.3.67";
    tp.try_add(code, kalpap);
    tp.try_add(code, deSya);
    tp.try_add(code, deSIyar);

    tp.with_context(Ajnate, |tp| {
        try_base_cases(tp, "5.3.73");
    });

    tp.with_context(Kutsite, |tp| {
        tp.optional_try_add("5.3.75", kan);
        try_base_cases(tp, "5.3.74");
    });

    tp.with_context(Anukampayam, |tp| {
        try_base_cases(tp, "5.3.76");
    });

    tp.with_context(Alpe, |tp| {
        try_base_cases(tp, "5.3.85");
    });

    tp.with_context(Hrasve, |tp| {
        let prati = tp.prati();
        if prati.has_text_in(&["kuwI", "SamI", "SuRqA"]) {
            // kuwIra, SamIra, SuRqAra
            tp.try_add("5.3.88", ra);
        } else if prati.has_text("kutU") {
            // kuwupa
            tp.try_add("5.3.89", qupac);
        } else if prati.has_text_in(&["kAsU", "goRI"]) {
            // kAsUtarI, goRItarI
            tp.try_add("5.3.90", zwarac);
        }
        tp.optional_try_add("5.3.87", kan);
        try_base_cases(tp, "5.3.86");
    });

    tp.with_context(Tanutve, |tp| {
        let prati = tp.prati();
        if prati.has_text_in(&["vatsa", "ukzan", "aSva", "fzaBa"]) {
            tp.try_add("5.3.91", zwarac);
        }
    });

    tp.with_context(Avakshepane, |tp| {
        tp.try_add("5.3.95", kan);
    });
}
