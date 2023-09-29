/*!
Runs rules that add krt-pratyayas after a dhAtu.

*Status: in progress, with support for major pratyayas.*


### Scope and role of krt-pratyayas

All of the rules that add a pratyaya after a dhAtu are defined within the scope of the adhikAra
rule 3.1.91 (dhAtoH), which extends to the end of 3.4. These pratyayas are of two types:

1. tiN-pratyayas, as well as the lakAras (lat, lit, lut) that those tiN-pratyayas replace. These
   create tinantas like gacCati, jagmuH, etc.

2. All other affixes, which are called *krt*-pratyayas. These create prAtipadikas, including
   nominal stems (gacCan, gata, gantavya) and indeclinables (gantum, gatvA).

Because *krt*-pratyayas create prAtipadikas, they can be followed by sup-pratyayas to create
subantas. However, the following krt-pratyayas are avyayas:

- any krt-pratyaya ending in "m", "e", "E", "o", or "O", per 1.1.39.
- ktvA, tosun, and kasun, per 1.1.40.

And after these krt-pratyayas, sup pratyayas cannot be added.

Once we add a krt-pratyaya, the derivation proceeds in the normal way. Most krt-pratyayas are
called `Ardhadhatuka` by rule 3.4.114, but a few that are `Sit` are called `Sarvadhatuka` by rule
3.4.113.


### Design

We want our code to answer two kinds of questions:

1. *Can this krt pratyaya apply to the given dhatu?* -- This is trivial for common pratyayas like
   `tavya`, but many pratyayas have complex exceptions that we must also check for. For an extreme
   example of this, see the implementation for 3.2.142.

2. *Given this dhatu, which krt pratyayas can we use with it?* -- We can answer this by running (1)
   for each of the 100+ krt-pratyayas we have, but a naive implementation could be unacceptably
   slow.

We focus on (1), not only because we expect this case to be more common but also -- and more
importantly -- because some of these rules have additional side effects beyond merely adding a
pratyaya. For example, 3.1.108 "hanas ta ca", which adds the pratyaya *kyap* while also changing
the last letter of the dhatu to *t*. Therefore, we cannot simply focus on (2) and return a list of
rules and pratyayas: with such an approach, modeling side effects is more cumbersome and less local.


### Internal API

We use a `KrtPrakriya` object that wraps `Prakriya` in a lightweight way and remembers which
pratyaya the caller wants to add. We add a pratyaya to `KrtPrakriya` only if both of the following
are true:

- There is a rule that allows the given `krt`.
- There is no rule that has blocked the given `krt`.

The main functions here are `try_add_krt` and `try_add_krt_for_tacchila_etc`, which each have their
own control flow. For details, please see the docs for those functions.
*/

use crate::args::Gana;
use crate::args::Krt;
use crate::args::Taddhita;
use crate::dhatu_gana as gana;
use crate::it_samjna;
use crate::krt::utils::KrtPrakriya;
use crate::operators as op;
use crate::prakriya::{Prakriya, Rule};
use crate::sounds::{s, Set};
use crate::stem_gana::TYAD_ADI;
use crate::tag::Tag as T;
use crate::term::Term;
use lazy_static::lazy_static;

lazy_static! {
    static ref AC: Set = s("ac");
    static ref IK: Set = s("ik");
    static ref PU: Set = s("pu~");
    static ref HAL: Set = s("hal");
    static ref EMPTY_TERM: Term = Term::make_text("");
}

fn is_nandi_grahi_pacadi(p: &KrtPrakriya, i: usize) -> bool {
    let dhatu = p.get(i).expect("should be present");

    // TODO: add the others.
    const NAND_ADI: &[&str] = &["nand", "jalp", "ram", "dfp"];

    #[allow(unused)]
    const GRAH_ADI: &[[&str; 2]] = &[
        ["", "grah"],
        ["ud", "sah"],
        ["ud", "das"],
        ["ud", "BAs"],
        ["", "sTA"],
        ["", "mantr"],
        ["sam", "mard"],
        ["ni", "rakz"],
        ["ni", "Sru"],
        ["ni", "vas"],
        ["ni", "vap"],
        ["ni", "SA"],
    ];

    const PAC_ADI: &[&str] = &[
        "pac", "vac", "vap", "vad", "cal", "tap", "pat", "nadaw", "Bazaw", "vas", "garat",
        "plavaw", "cIraw", "mAhaw", "jara", "mara", "kzara", "kzama", "sUdaw", "devaw", "moraw",
        "seva", "meza", "kroDa", "vraRa", "daMSa", "daSa", "damBa", "jAraBara", "Svapaca", "meGa",
        "kIza", "kzapa", "mada", "raja", "dIzaw", "caraw",
    ];

    // TODO: add the others.
    // TODO: at this length, maybe a set? sorted vec?
    dhatu.has_text_in(NAND_ADI) || dhatu.has_text_in(PAC_ADI)
}

fn try_add_upapada_krt(p: &mut Prakriya, krt: Krt) -> Option<bool> {
    use Krt::*;

    const DIVA_ADI: &[&str] = &[
        "divA", "viBA", "niSA", "praBA", "BAs", "kAra", "anta", "ananta", "Adi", "bahu", "nAndI",
        "kim", "lipi", "libi", "bali", "Bakti", "kartf", "citra", "kzetra", "jaNGA", "bAhu",
        "ahas", "yat", "tat", "Danus", "arus",
    ];
    const ADHYA_ADI: &[&str] = &[
        "AQya", "suBaga", "sTUla", "palita", "nagna", "anDa", "priya",
    ];

    // For convenience below, wrap `Prakriya` in a new `KrtPrakriya` type that contains `krt` and
    // records whether or not any of these rules were applied.
    let mut wrap = KrtPrakriya::new(p, krt);
    let i_dhatu = wrap.p.find_first_where(|t| t.is_dhatu())?;
    let dhatu = wrap.get(i_dhatu)?;

    let upapada = match wrap.p.get_if(0, |t| t.has_tag(T::Pratipadika)) {
        Some(t) => t,
        None => &EMPTY_TERM,
    };

    let nau = wrap.p.has(i_dhatu + 1, |t| t.has_u("Ric"));
    let upasarge = i_dhatu > 0 && wrap.p.has(i_dhatu - 1, |t| t.is_upasarga());

    match krt {
        aR | ka | ac | wa | wak => {
            if upapada.has_text_in(&["kzema", "priya", "madre"]) && dhatu.has_u("qukf\\Y") {
                // Also repeated for khac below.
                wrap.try_add("3.2.44", aR);
            } else if dhatu.has_u_in(&["hve\\Y", "ve\\Y", "mA\\N"]) {
                wrap.try_add("3.2.2", aR);
            } else if dhatu.has_u("zWA\\") {
                // Also 3.2.77
                wrap.try_add("3.2.4", ka);
            } else if dhatu.has_u("KyAY") {
                // gosaNKya
                wrap.try_add("3.2.7", ka);
            } else if !upasarge && dhatu.has_u("gE\\") {
                // for pA, see 3.2.8.v1 below.
                wrap.try_add("3.2.8", wak);
            } else if upapada.has_text_in(&["surA", "SIDu"])
                && !upasarge
                && dhatu.has_u("pA\\")
                && dhatu.has_gana(Gana::Bhvadi)
            {
                wrap.try_add("3.2.8.v1", wak);
            } else if dhatu.has_u("hf\\Y") {
                if wrap.has_upasarga(i_dhatu, "AN") {
                    // tAcCIlye
                    wrap.optional_try_add("3.2.11", ac);
                } else {
                    // an-udyamane, vayasi
                    wrap.optional_try_add("3.2.9", ac);
                }
            } else if dhatu.has_u("graha~^") {
                if upapada.has_text_in(&[
                    "Sakti", "lANgala", "aNkuSa", "yazwi", "tomara", "Gawa", "GaWI", "Danus",
                ]) {
                    wrap.try_add("3.2.9.v1", ac);
                } else if upapada.has_text("sUtra") {
                    // dhAri-arthe
                    wrap.optional_try_add("3.2.9.v2", ac);
                }
            } else if !upasarge && dhatu.has_antya('A') {
                wrap.try_add("3.2.3", ka);
            } else if wrap.has_upasarga_dhatu(i_dhatu, "pari", "mfjU~")
                || wrap.has_upasarga_dhatu(i_dhatu, "apa", "Ru\\da~^")
            {
                wrap.try_add("3.2.5", ka);
            } else if wrap.has_upasarga(i_dhatu, "pra") && dhatu.has_u_in(&["qudA\\Y", "jYA\\"]) {
                wrap.try_add("3.2.6", ka);
            } else if wrap.has_upasarga(i_dhatu, "AN") && dhatu.has_u("hf\\Y") {
                wrap.try_add("3.2.11", ac);
            } else if dhatu.has_u("arha~") {
                wrap.try_add("3.2.12", ac);
            } else if upapada.has_text("Sam") {
                wrap.try_add("3.2.14", ac);
            } else if dhatu.has_u("SIN") {
                wrap.try_add("3.2.15", ac);
            } else if dhatu.has_u("cara~") {
                if upapada.has_text_in(&["BikzA", "senA", "AdAya"]) {
                    wrap.try_add("3.2.17", wa);
                } else {
                    wrap.try_add("3.2.16", wa);
                }
            } else if dhatu.has_u("sf\\") {
                if upapada.has_text_in(&["puraH", "agrataH", "agre"]) {
                    wrap.try_add("3.2.18", wa);
                } else if upapada.has_text("pUrva") {
                    wrap.try_add("3.2.19", wa);
                }
            } else if dhatu.has_u("qukf\\Y") {
                if upapada.has_text_in(&DIVA_ADI) || upapada.has_tag(T::Sankhya) {
                    wrap.try_add("3.2.21", wa);
                } else if upapada.has_text("karman") {
                    wrap.optional_try_add("3.2.22", wa);
                } else if upapada.has_text_in(&[
                    "Sabda", "Sloka", "kalaha", "gATA", "vEra", "cAwu", "sUtra", "mantra", "pada",
                ]) {
                    wrap.p.step("3.2.23");
                } else {
                    wrap.try_add("3.2.20", wa);
                }
            } else if dhatu.has_u("ha\\na~") {
                if upapada.has_text_in(&["jAyA", "pati"]) {
                    // Sense is "lakshane".
                    wrap.optional_try_add("3.2.52", krt);
                } else if upapada.has_text_in(&["hastin", "kapAwa"]) {
                    // Sense is "shaktau".
                    wrap.optional_try_add("3.2.54", krt);
                } else if upapada.has_text_in(&["pARi", "tAqa"]) && krt == ka {
                    let sub = if upapada.has_text("pARi") {
                        "pARiGa"
                    } else {
                        "tAqaGa"
                    };
                    wrap.optional_do_nipatana("3.2.55", sub);
                } else {
                    // Sense is "a-manuSya-kartrke".
                    wrap.optional_try_add("3.2.53", krt);
                }
            }

            // (base case)
            if !wrap.has_krt {
                // kumBakAra, ...
                wrap.try_add("3.2.1", aR);
            }
        }

        in_ => {
            if upapada.has_text_in(&["stamba", "Sakft"]) && dhatu.has_u("qukf\\Y") {
                wrap.try_add("3.2.24", krt);
            } else if dhatu.has_u("hf\\Y") {
                if upapada.has_text_in(&["dfti", "nATa"]) {
                    wrap.try_add("3.2.25", krt);
                }
            } else if (upapada.has_text("Pala") && dhatu.has_u("graha~^"))
                || (upapada.has_text("Atman") && dhatu.has_u("quBf\\Y"))
            {
                let sub = if upapada.has_text("Pala") {
                    "Palegrahi"
                } else {
                    "AtmamBari"
                };
                wrap.do_nipatana("3.2.26", sub);
            }
        }
        KaS => {
            let nasika = upapada.has_text("nAsikA");
            let stana = upapada.has_text("stana");
            let dhma = dhatu.has_u("DmA\\");
            let dhe = dhatu.has_u("De\\w");
            if dhatu.has_text("ej") && nau {
                wrap.try_add("3.2.28", krt);
            } else if (nasika && (dhma || dhe)) || (stana && dhe) {
                wrap.try_add("3.2.29", krt);
            } else if (dhma || dhe) && upapada.has_text_in(&["nAqI", "muzwi"]) {
                wrap.try_add("3.2.30", krt);
            } else if upapada.has_text("kUla")
                && wrap.has_upasarga(i_dhatu, "ud")
                && dhatu.has_u_in(&["ru\\jo~", "va\\ha~^"])
            {
                wrap.try_add("3.2.31", krt);
            } else if upapada.has_text_in(&["vaha", "aBra"]) && dhatu.has_u("li\\ha~^") {
                wrap.try_add("3.2.32", krt);
            } else if dhatu.has_u("qupa\\ca~^z") {
                if upapada.has_text_in(&["mita", "naKa"]) {
                    wrap.try_add("3.2.34", krt);
                } else {
                    wrap.try_add("3.2.33", krt);
                }
            } else if upapada.has_text_in(&["viDu", "arus"]) && dhatu.has_u("tu\\da~^") {
                wrap.try_add("3.2.35", krt);
            } else if (upapada.has_text("asUrya") && dhatu.has_u("df\\Si~r"))
                || (upapada.has_text("lalAwa") && dhatu.has_u("ta\\pa~"))
            {
                wrap.try_add("3.2.36", krt);
            } else if (upapada.has_text("ugra") && dhatu.has_u("df\\Si~r"))
                || (upapada.has_text("irA") && dhatu.has_u("madI~"))
                || (upapada.has_text("pARi") && dhatu.has_u("DmA\\"))
            {
                let sub = if upapada.has_text("ugra") {
                    "ugrampaSya"
                } else if upapada.has_text("irA") {
                    "irammada"
                } else {
                    "pARinDama"
                };
                wrap.do_nipatana("3.2.37", sub);
            } else if dhatu.has_u("ma\\na~\\") {
                wrap.try_add("3.2.83", krt);
            }
        }

        Kac => {
            if upapada.has_text_in(&["priya", "vaSa"]) && dhatu.has_u("vada~") {
                wrap.try_add("3.2.38", krt);
            } else if upapada.has_text_in(&["dvizat", "para"]) && dhatu.has_text("tAp") && nau {
                wrap.try_add("3.2.39", krt);
            } else if upapada.has_text("vAc") && dhatu.has_text("yam") {
                wrap.try_add("3.2.40", krt);
            } else if upapada.has_text_in(&["pur", "sarva"])
                && ((dhatu.has_text("dAr") && nau) || dhatu.has_text("sah"))
            {
                wrap.try_add("3.2.41", krt);
            } else if upapada.has_text_in(&["sarva", "kUla", "aBra", "karIza"])
                && dhatu.has_text("kaz")
            {
                wrap.try_add("3.2.42", krt);
            } else if dhatu.has_u("qukf\\Y") {
                if upapada.has_text_in(&["meGa", "fti", "Baya"]) {
                    wrap.try_add("3.2.43", krt);
                } else if upapada.has_text_in(&["kzema", "priya", "madra"]) {
                    wrap.try_add("3.2.44", krt);
                }
            } else if upapada.has_text("ASita") && dhatu.has_u("BU") {
                wrap.try_add("3.2.45", krt);
            }
        }

        qa => {
            if dhatu.has_u("ga\\mx~") {
                if upapada.has_text_in(&[
                    "anta", "atyanta", "aDvan", "dUra", "pAra", "sarva", "ananta",
                ]) {
                    wrap.try_add("3.2.48", krt);
                }
            } else if dhatu.has_u("ha\\na~") {
                if upapada.has_text_in(&["kleSa", "tamas"]) && wrap.has_upasarga(i_dhatu, "apa") {
                    wrap.try_add("3.2.50", krt);
                } else {
                    wrap.try_add("3.2.49", krt);
                }
            } else if dhatu.has_u("janI~\\") {
                // TODO: upapada
                wrap.try_add("3.2.97", krt);
            }
        }

        Rini => {
            if upapada.has_text_in(&["kumAra", "Sirza"]) && dhatu.has_u("ha\\na~") {
                wrap.try_add("3.2.51", krt);
            } else if upapada.has_text("vrata") {
                wrap.try_add("3.2.80", krt);
            } else if dhatu.has_u("ma\\na~\\") {
                wrap.try_add("3.2.82", krt);
            } else if dhatu.has_u("ya\\ja~^") {
                wrap.try_add("3.2.85", krt);
            } else if dhatu.has_u("ha\\na~") {
                wrap.try_add("3.2.86", krt);
            } else {
                wrap.try_add("3.2.78", krt);
            }
        }

        Kyun => {
            if upapada.has_text_in(&ADHYA_ADI) && dhatu.has_u("qukf\\Y") {
                wrap.try_add("3.2.56", krt);
            }
        }

        KizRuc | KukaY => {
            if upapada.has_text_in(&ADHYA_ADI) && dhatu.has_u("BU") {
                wrap.try_add("3.2.57", krt);
            }
        }

        kvin | kaY => {
            if upapada.has_text_in(TYAD_ADI) && dhatu.has_u("df\\Si~r") {
                wrap.try_add("3.2.60", krt);
            } else if krt == kvin {
                if !upapada.has_text("udaka") && dhatu.has_text("spfS") {
                    wrap.try_add("3.2.58", kvin);
                } else {
                    let code = "3.2.59";
                    if upapada.has_text("ftu") && dhatu.has_u("ya\\ja~^") {
                        wrap.do_nipatana(code, "ftvij");
                    } else if dhatu.has_u("YiDfzA~") {
                        wrap.do_nipatana(code, "daDfz");
                    } else if dhatu.has_u("sf\\ja~") {
                        wrap.do_nipatana(code, "sraj");
                    } else if dhatu.has_u("di\\Sa~^") {
                        wrap.do_nipatana(code, "diS");
                    } else if wrap.has_upasarga(i_dhatu, "ud") && dhatu.has_u("zRi\\ha~") {
                        wrap.do_nipatana(code, "uzRih");
                    } else if dhatu.has_u("ancu~") {
                        wrap.p.op_term(code, i_dhatu, |t| {
                            t.set_text("aYc");
                            t.add_tag(T::Krt);
                        });
                        // HACK: update bookkeeping here.
                        wrap.has_krt = true;
                        wrap.tried = true;
                    } else if dhatu.has_u("yu\\ji~^r") {
                        wrap.do_nipatana(code, "yuj");
                    } else if dhatu.has_u("krunca~") {
                        wrap.do_nipatana(code, "kruYc");
                    }
                }
            }
        }

        viw => {
            if dhatu.has_u("a\\da~") {
                if upapada.has_text("kravya") {
                    wrap.try_add("3.2.69", krt);
                } else if !upapada.has_text("anna") {
                    wrap.try_add("3.2.68", krt);
                }
            }
        }

        kap => {
            if dhatu.has_u("du\\ha~^") {
                wrap.try_add_with("3.2.70", krt, |p, i| {
                    p.set(i, |t| t.set_antya("G"));
                });
            }
        }

        kvip => {
            if dhatu.has_u("ha\\na~") {
                if upapada.has_text_in(&["brahman", "BrURa", "vftra"]) {
                    wrap.try_add("3.2.87", krt);
                }
            } else if dhatu.has_u("zWA\\") {
                wrap.try_add("3.2.77", krt);
            } else if upapada.has_text("soma") && dhatu.has_u("zu\\Y") {
                wrap.try_add("3.2.90", krt);
            } else if upapada.has_text("agni") && dhatu.has_u("ci\\Y") {
                wrap.try_add("3.2.91", krt);
            } else {
                wrap.try_add("3.2.76", krt);
            }
        }

        ini => {
            if wrap.has_upasarga(i_dhatu, "vi") && dhatu.has_u("qukrI\\Y") {
                if upapada.has_text("DAnya") {
                    wrap.p.step(Rule::Kashika("3.2.93"));
                } else {
                    wrap.try_add("3.2.93", krt);
                }
            }
        }

        kvanip => {
            if dhatu.has_u("df\\Si~r") {
                wrap.try_add("3.2.94", krt);
            } else if dhatu.has_u_in(&["qukf\\Y", "yu\\Da~\\"]) {
                if upapada.has_text("rAjan") {
                    wrap.try_add("3.2.95", krt);
                } else if upapada.has_text("saha") {
                    wrap.try_add("3.2.96", krt);
                }
            }
        }
        _ => {}
    }

    Some(wrap.has_krt)
}

/// Runs rules that try to add `krt` in one of three specific senses. (3.2.134 - 3.2.179)
///
/// The three senses of these pratyayas are:
/// - tacchIla (nature)
/// - taddharma (duty)
/// - tatsAdhukAra (ability)
///
///
/// ### Control flow
///
/// In this section, we ignore rule 3.1.94, which allows multiple pratyayas to optionally apply:
///
/// > ṇvulaiva siddhe vuñvidhānaṃ jñāpanārthaṃ, tācchīlikeṣu vā'sarūpanyāyena tṛjādayo na bhavantīti
/// > -- Kashika vrtti on 3.2.146
///
/// > tacchīlādiṣu vā'sarūpavidhirnāstīti। tenālaṅkārakaḥ, nirākartetyādīnāṃ tacchīlādiṣu sādhutvaṃ
/// > na bhavati॥
/// > -- Nyasa on 3.2.146
///
/// Therefore, the pratyayas in this section are blocking: for a given dhatu, exactly one of these
/// pratyayas can be used.
///
/// That said, there are some minor exceptions here and there where multiple pratyayas can apply.
fn try_add_krt_for_tacchila_etc(p: &mut KrtPrakriya, i: usize, krt: Krt) -> Option<()> {
    use Krt as K;

    // `i` is the index of the last item called dhatu.
    let dhatu = p.get(i)?;

    let prev = if i > 0 { p.get(i - 1) } else { None };
    let upasarge = prev.map_or(false, |t| t.has_tag(T::Upasarga));

    let has_prefix_and_text = |x, y| p.has_prefix(x) && dhatu.has_text(y);
    let has_nic = p.p.terms().last()?.has_u("Ric");
    let has_yan = p.p.terms().last()?.has_u("yaN");
    let has_san = p.p.terms().last()?.has_u("san");

    if has_prefix_and_text("alam", "kf")
        || (p.has_prefixes(&["nis", "A"]) && dhatu.has_text("kf"))
        || has_prefix_and_text("pra", "jan")
        || has_prefix_and_text("ud", "pac")
        || has_prefix_and_text("ud", "pat")
        || has_prefix_and_text("ud", "mad")
        || dhatu.has_text("ruc")
        || has_prefix_and_text("apa", "trap")
        || dhatu.has_text_in(&["vft", "vfD", "sah", "car"])
    {
        p.try_add("3.2.136", K::izRuc);
    } else if dhatu.has_text_in(&["glE", "ji", "sTA", "BU"]) {
        p.try_add("3.2.139", K::ksnu);
    } else if dhatu.has_text_in(&["tras", "gfD", "Dfz", "kzip"]) {
        p.try_add("3.2.140", K::knu);
    } else if dhatu.has_u_in(gana::SHAM_ADI) {
        p.try_add("3.2.141", K::GinuR);
    } else if has_prefix_and_text("sam", "pfc")
        || has_prefix_and_text("anu", "ruD")
        || has_prefix_and_text("A", "yam")
        || has_prefix_and_text("A", "yas")
        || has_prefix_and_text("pari", "sf")
        || has_prefix_and_text("sam", "sfj")
        || has_prefix_and_text("pari", "dev")
        || has_prefix_and_text("sam", "jvar")
        || has_prefix_and_text("pari", "kzip")
        || has_prefix_and_text("pari", "raw")
        || has_prefix_and_text("pari", "vad")
        || has_prefix_and_text("pari", "dah")
        || has_prefix_and_text("pari", "muh")
        || dhatu.has_text_in(&["duz", "dviz", "druh", "duh", "yuj"])
        || has_prefix_and_text("A", "krIq")
        || has_prefix_and_text("vi", "vij")
        || dhatu.has_text_in(&["tyaj", "ranj", "Baj"])
        || has_prefix_and_text("ati", "car")
        || has_prefix_and_text("apa", "car")
        || has_prefix_and_text("A", "muz")
        || (p.has_prefixes(&["aBi", "A"]) && dhatu.has_text("han"))
    {
        p.try_add("3.2.142", K::GinuR);
    } else if p.has_prefix("vi") && dhatu.has_text_in(&["kaz", "las", "katT", "sranB"]) {
        p.try_add("3.2.143", K::GinuR);
    } else if p.has_prefix_in(&["vi", "apa"]) && dhatu.has_text("laz") {
        p.try_add("3.2.144", K::GinuR);
    } else if p.has_prefix("pra")
        && dhatu.has_u_in(&["lapa~", "sf\\", "dru\\", "maTe~", "vada~", "va\\sa~"])
    {
        p.try_add("3.2.145", K::GinuR);
    }

    // Break the `if` chain so that pari-kzip and pari-raw can match again here.
    let dhatu = p.get(i)?;
    let has_prefix_and_text = |x, y| p.has_prefix(x) && dhatu.has_text(y);
    if dhatu.has_text_in(&["nind", "hins", "kliS", "KAd"])
        || (has_prefix_and_text("vi", "naS") && has_nic)
        || has_prefix_and_text("pari", "kzip")
        || has_prefix_and_text("pari", "raw")
        || (has_prefix_and_text("pari", "vad") && has_nic)
        || (p.has_prefixes(&["vi", "A"]) && dhatu.has_text("BAz"))
        // TODO: not in dhatupatha -- how does the prakriya start?
        || dhatu.has_text("asUy")
    {
        p.try_add("3.2.146", K::vuY);
    } else if upasarge && dhatu.has_text_in(&["div", "kruS"]) {
        p.try_add("3.2.147", K::vuY);
    } else if dhatu.has_text_in(&[
        "laz", "pat", "pad", "sTA", "BU", "vfz", "han", "kam", "gam", "SF",
    ]) {
        p.try_add("3.2.154", K::ukaY);
    } else if dhatu.has_text_in(&["jalp", "Bikz", "kuww", "lunw", "vf"]) {
        p.try_add("3.2.154", K::zAkan);
    } else if dhatu.has_text_in(&["sf", "Gas", "ad"]) {
        p.try_add("3.2.160", K::kmarac);
    } else if dhatu.has_text_in(&["Banj", "BAs", "mid"]) {
        p.try_add("3.2.161", K::Gurac);
    } else if (dhatu.has_u("vida~") && dhatu.has_gana(Gana::Adadi))
        || dhatu.has_text_in(&["Bid", "Cid"])
    {
        // Per commentaries, allow only this specific `vid`.
        p.try_add("3.2.162", K::kurac);
    } else if dhatu.has_u("i\\R") || dhatu.has_text_in(&["naS", "ji", "sf"]) {
        p.try_add("3.2.163", krt);
    } else if dhatu.has_text("jAgf") {
        p.try_add("3.2.165", K::Uka);
    } else if dhatu.has_text_in(&["yaj", "jap", "daS"]) && has_yan {
        p.try_add("3.2.166", K::Uka);
    } else if dhatu.has_text_in(&["nam", "kanp", "smi", "jas", "kam", "hins", "dIp"]) {
        p.try_add("3.2.167", K::ra);
    } else if has_san || has_prefix_and_text("A", "Sans") || dhatu.has_text("Bikz") {
        p.try_add("3.2.168", K::u);
    } else if dhatu.has_text_in(&["svap", "tfz"]) {
        p.try_add("3.2.172", K::najiN);
    } else if dhatu.has_text_in(&["Sf", "vand"]) {
        p.try_add("3.2.173", K::Aru);
    } else if dhatu.has_text("BI") {
        if krt == K::kruka {
            p.try_add("3.2.174.v1", K::kruka);
        } else if krt == K::kru {
            p.try_add("3.2.174", K::kru);
        } else {
            p.try_add("3.2.174", K::klukan);
        }
    } else if dhatu.has_text_in(&["sTA", "IS", "BAs", "pis", "kas"]) {
        p.try_add("3.2.175", K::varac);
    } else if i > 0 && p.p.has(i - 1, |t| t.has_text("yA")) && has_yan {
        // yAyAvara
        p.try_add("3.2.176", K::varac);
    } else if dhatu.has_text_in(&["BrAj", "BAs", "Durv", "dyut", "Urj", "pF", "ju"]) {
        // TODO: grAva-stut
        p.try_add("3.2.177", K::kvip);
    } else if dhatu.has_text_in(&["yuj", "Cid", "Bid"]) {
        // anyebhyo 'pi dRzyate -- so, include what the commentators mention.
        p.try_add("3.2.178", K::kvip);
    }

    if !p.tried {
        p.try_add("3.2.135", K::tfn);
    }

    Some(())
}

/// Runs rules that try to add the given `krt` suffix to the dhatu. (3.1.91 - 3.4.76)
///
///
/// ### API
///
/// This function receives a `Prakriya` and the `krt` suffix the caller wishes to add. It returns
/// one of the following vowels:
///
/// - `Some(true)`, which denotes that `krt` was successfully added to the prakriya.
/// - `Some(false)`, which denotes that there is no rule that can add `krt` to the prakriya.
/// - `None`, which denotes that the prakriya is ineligible to receive a krt pratyaya, e.g. if the
///    prakriya does not contain a dhatu.
///
///
/// ### Control flow
///
/// Most of the rules here are constrained by the following paribhasha:
///
/// > 3.1.94 vā́sarūpo'striyām
///
/// In plain English: the pratyayas introduced from 3.1.95 up to 3.3.94 (striyām ktiṅ) block each
/// other if they have the same form (sarūpa). Otherwise, there is no blocking and each rule is
/// optional (vā).
///
/// For example, if one rule adds *ghañ* and another adds *ap*, both pratyayas have the same form
/// ("a") if their it letters are removed, and therefore the *ap* rule will block the *ghañ* rule.
///
/// For details, see: <https://ashtadhyayi.com/sutraani/3/1/94>
fn try_add_krt(p: &mut Prakriya, krt: Krt) -> Option<bool> {
    use Krt as K;

    let i = p.find_last(T::Dhatu)?;
    let prev = if i > 0 { p.get(i - 1) } else { None };

    // Pre-calculate some common properties.
    let upasarge = prev.map_or(false, |t| t.has_tag(T::Upasarga));
    let supi = prev.map_or(false, |t| t.has_tag(T::Sup));

    // For convenience below, wrap `Prakriya` in a new `KrtPrakriya` type that contains `krt` and
    // records whether or not any of these rules were applied.
    let mut wrap = KrtPrakriya::new(p, krt);
    let dhatu = wrap.get(i)?;

    match krt {
        // ------------------------------------------
        // krtyAH
        // ------------------------------------------
        K::tavyat | K::tavya | K::anIyar => {
            let added = wrap.try_add("3.1.96", krt);
            if added && krt == K::tavyat && wrap.get(i)?.has_u("va\\sa~") {
                // vAstavya
                wrap.p
                    .op_optional("3.1.96.v1", op::t(i + 1, |t| t.add_tag(T::Rit)));
            }
        }

        K::kelimar => {
            wrap.try_add("3.1.96.v2", krt);
        }

        // "ya" (3.1.97 - 3.1.132)
        K::yat | K::kyap | K::Ryat => {
            // TODO: nipatana:
            // - 3.1.101 - 3.1.105
            // - 3.1.114 - 3.1.119
            // - 3.1.121 - 3.1.123
            // - 3.1.127 - 3.1.132
            let vapi_rapi = &["vap", "rap", "lap", "trap", "cam"];

            // If true, skip 3.1.110.
            let mut skip_3_1_110 = false;

            // Specific rules (optional)
            let dhatu = wrap.get(i)?;
            if dhatu.has_u("fca~") {
                // ṛdupadhādapi ṛcerata eva nipātanāt ṇyat bhavati
                wrap.try_add("7.3.66", K::Ryat);
            } else if dhatu.has_u("quBf\\Y") {
                wrap.optional_try_add("3.1.112", K::kyap);
            } else if dhatu.has_u_in(&["qukf\\Y", "vfzu~"]) {
                if dhatu.has_text("kf") {
                    wrap.optional_try_add("3.1.120", K::kyap);
                } else {
                    // This rule makes rule 3.1.110 optional for vfz.
                    skip_3_1_110 = wrap.p.op_optional("3.1.120", |_| {});
                }
            } else if dhatu.has_text("mfj") {
                // This rule makes rule 3.1.110 optional for mfj.
                skip_3_1_110 = wrap.p.op_optional("3.1.113", |_| {});
            }

            // Specific rules (required)
            let dhatu = wrap.get(i)?;
            if dhatu.has_u_in(&["Sa\\kx~", "zaha~\\"]) {
                wrap.try_add("3.1.99", K::yat);
            } else if dhatu.has_u_in(&["gada~", "madI~", "cara~", "ya\\ma~"]) && !upasarge {
                wrap.try_add("3.1.100", K::yat);
            } else if dhatu.has_u("cara~") && i > 0 && wrap.get(i - 1)?.has_u("AN") {
                wrap.optional_try_add("3.1.100.v1", K::yat);
            } else if !upasarge && dhatu.has_text("vad") && supi {
                wrap.try_add("3.1.106", K::yat);
                wrap.try_add("3.1.106", K::kyap);
            } else if !upasarge && supi && dhatu.has_u("BU") {
                // The condition here is bhAve, but per the kashika, it is always bhAve per 3.4.70,
                // so it is effectively nitya.
                wrap.try_add("3.1.107", K::kyap);
            } else if !upasarge && supi && dhatu.has_u("ha\\na~") {
                wrap.try_add_with("3.1.108", K::kyap, |p, i| {
                    p.set(i, |t| t.set_antya("t"));
                });
            } else if dhatu.has_u("i\\R") || dhatu.has_text_in(&["stu", "SAs", "vf", "df", "juz"]) {
                wrap.try_add("3.1.109", K::kyap);
            } else if dhatu.has_upadha('f') && !dhatu.has_text_in(&["kfp", "cft"]) && !skip_3_1_110
            {
                wrap.try_add("3.1.110", K::kyap);
            } else if dhatu.has_text("Kan") {
                wrap.try_add_with("3.1.111", K::kyap, |p, i| {
                    p.set(i, |t| t.set_antya("I"));
                });
            } else if (wrap.has_prefix("A") && dhatu.has_text("su")) || dhatu.has_text_in(vapi_rapi)
            {
                wrap.try_add("3.1.126", K::Ryat);
            }

            // General rules (optional)
            let dhatu = wrap.get(i)?;
            if dhatu.has_antya('u') || dhatu.has_antya('U') {
                wrap.optional_try_add("3.1.125", K::Ryat);
            } else if dhatu.has_u("ha\\na~") {
                wrap.optional_try_add_with("3.1.97.v2", K::yat, |p, i| {
                    p.set(i, |t| t.set_text("vaD"));
                });
            }

            // General rules (obligatory)
            let dhatu = wrap.get(i)?;
            if !wrap.tried {
                if dhatu.has_upadha('a') && dhatu.has_antya(&*PU) {
                    // Sapya, laBya, japya
                    wrap.try_add("3.1.98", K::yat);
                } else if dhatu.has_u_in(&["taka~", "Sasu~\\", "cate~^", "yatI~\\", "janI~\\"]) {
                    wrap.try_add("3.1.97.v1", K::yat);
                } else if dhatu.has_antya('f') || dhatu.has_antya('F') || dhatu.has_antya(&*HAL) {
                    wrap.try_add("3.1.124", K::Ryat);
                } else if dhatu.has_antya(&*AC) {
                    wrap.try_add("3.1.97", K::yat);
                }
            }
        }

        K::Rvul | K::tfc => {
            wrap.try_add("3.1.133", krt);
        }

        K::lyu | K::Rini => {
            if is_nandi_grahi_pacadi(&wrap, i) {
                wrap.try_add("3.1.134", krt);
            } else if krt == K::Rini {
                // TODO: supi
                wrap.try_add("3.2.78", krt);
            }
        }

        // "a" (3.1.134 - ???)
        K::ac | K::Sa | K::ka | K::Ra => {
            // These are all bhvAdi dhAtus, so also check for `Bhvadi` to avoid other dhatus.
            let pa_ghra = &["pA\\", "GrA\\", "DmA\\", "De\\w", "df\\Si~r"];

            if upasarge && dhatu.has_u_in(pa_ghra) && dhatu.has_gana(Gana::Bhvadi) {
                wrap.try_add("3.1.137", K::Sa);
            } else if dhatu.has_u_in(&[
                "li\\pa~^", "vi\\dx~^", "pF", "vida~", "cita~", "sAti", "zaha~\\",
            ]) {
                wrap.try_add("3.1.138", K::Sa);
            } else if dhatu.has_upadha(&*IK) || dhatu.has_u_in(&["JYA\\", "prI\\Y", "kF"]) {
                // vikzipa, viliKa, buDa
                wrap.try_add("3.1.135", K::ka);
            } else if upasarge && dhatu.has_antya('A') {
                wrap.try_add("3.1.136", K::ka);
            } else if krt == K::ac {
                // ajvidhiḥ sarvadhātubhyaḥ paṭhyante ca pacādayaḥ। aṇbādhanārtham eva
                // syāt sidhyanti śvapacādayaḥ।
                wrap.try_add(Rule::Kashika("3.1.134"), K::ac);
            } else if dhatu.has_u_in(&["wudu\\", "RI\\Y"]) && !upasarge {
                wrap.try_add("3.1.142", K::Ra);
            } else if dhatu.has_u("graha~^") {
                wrap.try_add("3.1.144", K::ka);
            }
        }

        K::zvun | K::vun => {
            if dhatu.has_text_in(&["nft", "Kan", "ranj"]) {
                wrap.try_add_with("3.1.145", K::zvun, |p, i| {
                    if p.has(i, |t| t.has_text("ranj")) {
                        // per kashika
                        p.set(i, |t| t.set_upadha(""));
                    }
                });
            } else if dhatu.has_text_in(&["pru", "sf", "lU"]) {
                wrap.try_add("3.1.149", K::vun);
            }

            // Allowed for all dhatus in the sense of ASIH.
            wrap.try_add("3.1.150", K::vun);
        }

        K::Takan => {
            if dhatu.has_text("gE") {
                wrap.try_add("3.1.146", krt);
            }
        }

        K::Ryuw => {
            if dhatu.has_text("gE") {
                wrap.try_add("3.1.146", krt);
            } else if dhatu.has_text("hA") {
                wrap.try_add("3.1.148", krt);
            }
        }

        K::kvip => {
            if dhatu.has_text_in(&[
                "sad", "sU", "dviz", "druh", "duh", "yuj", "vid", "Bid", "Cid", "ji", "nI", "rAj",
            ]) {
                // Exclude some of the dhatus above:
                //
                // > sū iti dviṣā sāhacaryāt sūteḥ ādādikasya grahaṇaṃ, na suvateḥ taudādikasya।
                // > yujiryoge, yuja samādhau, dvayorapi grahaṇam। vida jñāne, vida sattāyām, vida
                // > vicāraṇe, trayāṇāmapi grahaṇam। na lābhārthasya videḥ, akārasya vivakṣatatvāt
                // -- KV
                let skip = (dhatu.has_text("sU") && !dhatu.has_gana(Gana::Adadi))
                    || (dhatu.has_text("vid") && dhatu.has_tag(T::xdit));
                if !skip {
                    wrap.try_add("3.2.61", krt);
                }
            } else {
                wrap.try_add("3.2.76", krt);
            }
        }

        K::Rvi => {
            if dhatu.has_u("Ba\\ja~^") {
                wrap.try_add("3.2.62", krt);
            }
        }

        K::manin | K::kvanip | K::vanip | K::vic => {
            let code = "3.2.75";
            if krt == K::manin && dhatu.has_text("Bas") {
                wrap.try_add(code, krt);
            } else if krt == K::vic && dhatu.has_text("riz") {
                wrap.try_add(code, krt);
            }
        }

        K::kta | K::ktavatu => {
            if dhatu.has_tag(T::YIt) {
                wrap.try_add("3.2.187", Krt::kta);
            }
            wrap.try_add("3.2.102", krt);

            if wrap.has_krt {
                let i_last = wrap.p.terms().len() - 1;
                wrap.p.op_term("1.1.26", i_last, op::add_tag(T::Nistha));
            }
        }

        K::Nvanip => {
            if dhatu.has_u("zu\\Y") || dhatu.has_text("yaj") {
                wrap.try_add("3.2.103", krt);
            }
        }

        K::atfn => {
            if dhatu.has_text("jF") {
                wrap.try_add("3.2.104", krt);
            }
        }

        K::kAnac | K::kvasu => {
            // TODO: does this check make sense for kvasu and kAnac? Or should we relax this check
            // because these are mainly chAndasa? For now, disable it.
            /*
            let has_pada_match = match krt {
                K::kvasu => wrap.p.has_tag(T::Parasmaipada),
                // taṅānāv ātmanepadam (1.4.100)
                K::kAnac => wrap.p.has_tag(T::Atmanepada),
                _ => false,
            };
            */

            // Although the rule has "chandasi" by anuvrtti from 3.2.105, kvasu~ has wide
            // application:
            //
            // > kavayastu bahulaṃ prayuñjate । taṃ tasthivāṃsaṃ nagaropakaṇṭhe iti । śreyāṃsi
            // > sarvāṇyadhijagmuṣaste ityādi ॥
            // -- Siddhanta Kaumudi on 3.2.107.
            if !wrap.has_krt {
                let i_la = wrap.p.terms().len() - 1;
                if krt == K::kvasu && dhatu.has_text_in(&["sad", "vas", "Sru"]) {
                    wrap.try_replace_lakara("3.2.108", i_la, krt);
                } else {
                    wrap.try_replace_lakara("3.2.107", i_la, krt);
                }
            }
        }

        // 3.2.111 - 3.2.123 are various lakaras.
        // TODO: 3.3.130 - 3.2.133
        K::Satf | K::SAnac => {
            let has_pada_match = match krt {
                K::Satf => wrap.p.has_tag(T::Parasmaipada),
                // taṅānāv ātmanepadam (1.4.100)
                K::SAnac => wrap.p.has_tag(T::Atmanepada),
                _ => false,
            };

            /*
            // TODO: not sure if these block 3.2.127 below.
            if krt == K::Satf {
                if dhatu.has_text("dviz") {
                    wrap.try_add("3.2.131", krt);
                } else if dhatu.has_u("zu\\Y") {
                    wrap.try_add("3.2.132", krt);
                } else if dhatu.has_text("arh") {
                    wrap.try_add("3.2.133", krt);
                }
            }
            */

            // 3.2.125 and 3.2.126 define other semantics conditions for Satf and SAnac.
            if has_pada_match && !wrap.has_krt {
                let i_la = wrap.p.terms().len() - 1;
                wrap.try_replace_lakara("3.2.128", i_la, krt);
                wrap.p.op_term("3.2.127", i_la, op::add_tag(T::Sat));
            }
        }

        // These are treated separately from SAnac because they are not `Lat` replacements.
        K::SAnan | K::cAnaS => {
            if dhatu.has_text_in(&["pU", "yaj"]) {
                wrap.try_add("3.2.128", K::SAnan);
            } else {
                wrap.try_add("3.2.129", K::cAnaS);
            }
        }

        // Rules 3.2.134 - 3.2.179 have a separate control flow and are defined in
        // `try_add_krt_for_tacchila_etc`.

        // The normal control flow resumes from 3.2.180 onward.
        K::qu => {
            if wrap.has_prefix_in(&["vi", "pra", "sam"]) && dhatu.has_text("BU") {
                wrap.try_add("3.2.180", krt);
            }
        }

        K::zwran => {
            // TODO: move others over
            const DAP_ADI: &[&str] = &[
                "dA\\p",
                "RI\\Y",
                "Sasu~",
                "yu",
                "yu\\ji~^r",
                "zwu\\Y",
                "tu\\da~^",
                "zi\\Y",
                "zi\\ca~^",
                "mi\\ha~",
                "da\\nSa~",
                "Ra\\ha~^",
            ];
            if dhatu.has_u("De\\w") {
                wrap.try_add("3.2.181", krt);
            } else if dhatu.has_u_in(DAP_ADI) {
                wrap.try_add("3.2.182", krt);
            } else if dhatu.has_text("pU") {
                wrap.try_add("3.2.183", krt);
            }
        }

        K::itra => {
            if dhatu.has_text_in(&["f", "lU", "DU", "sU", "Kan", "sah", "car"]) {
                wrap.try_add("3.2.184", krt);
            } else if dhatu.has_text("pU") {
                // Also allowed by 3.2.186.
                wrap.try_add("3.2.185", krt);
            }
        }

        K::tumun => {
            wrap.try_add("3.3.10", krt);
        }

        K::GaY => {
            if wrap.has_prefix("ava") && dhatu.has_u_in(&["tF", "stFY"]) {
                wrap.try_add("3.3.120", K::GaY);
            } else {
                // TODO: move with other a-pratyayas?
                let dhatu = wrap.get(i)?;
                if dhatu.has_text_in(&["pad", "ruj", "viS", "spfS"]) {
                    wrap.try_add("3.3.16", K::GaY);
                } else if dhatu.has_text("sf") {
                    wrap.try_add("3.3.17", K::GaY);
                } else {
                    wrap.try_add("3.3.18", K::GaY);
                }
            }
        }

        K::ap => {
            if dhatu.has_u("aja~") && i > 0 && wrap.p.has(i - 1, |t| t.has_u_in(&["sam", "ud"])) {
                wrap.try_add("3.3.69", krt);
            }
        }

        K::ktri => {
            if dhatu.has_tag(T::qvit) {
                if wrap.try_add("3.3.88", krt) {
                    // TODO: put this somewhere else?
                    wrap.p.op("4.4.20", |p| {
                        p.push(Taddhita::map.to_term());
                    });
                    it_samjna::run(wrap.p, i + 2).expect("should never fail");
                }
            }
        }

        K::aTuc => {
            if dhatu.has_tag(T::wvit) {
                wrap.try_add("3.3.89", krt);
            }
        }

        K::naN => {
            if dhatu.has_text_in(&["yaj", "yAc", "yat", "viC", "praC", "rakz"]) {
                wrap.try_add("3.3.90", krt);
            }
        }

        K::nan => {
            if dhatu.has_u("Yizva\\pa~") {
                wrap.try_add("3.3.91", krt);
            }
        }

        // ------------------------------------------
        // striyAm
        // ------------------------------------------
        K::ktin => {
            wrap.try_add_with("3.3.94", krt, |p, _| p.add_tag(T::Stri));
        }

        K::a => {
            if wrap.p.has(i, |t| t.is_pratyaya()) {
                wrap.try_add_with("3.3.102", krt, |p, _| p.add_tag(T::Stri));
            } else if dhatu.is_guru() && dhatu.has_antya(&*HAL) {
                wrap.try_add_with("3.3.103", krt, |p, _| p.add_tag(T::Stri));
            }
        }

        K::yuc => {
            if dhatu.has_u_in(&["Ric", "Asa~\\", "SranTa~"]) {
                wrap.try_add_with("3.3.107", krt, |p, _| p.add_tag(T::Stri));
            }
        }

        K::Rvuc => {}

        K::ani => {}

        K::lyuw => {
            wrap.try_add("3.3.115", krt);
        }

        K::Ga => {
            wrap.try_add("3.3.118", krt);
        }

        K::Kal => {
            // TODO: restrict
            wrap.try_add("3.3.126", krt);
        }

        K::ktic => {
            wrap.try_add("3.3.174", krt);
        }
        K::kamul | K::Ramul => {
            if krt == K::kamul {
                wrap.try_add("3.4.12", krt);
            } else {
                // TODO: move this rule somewhere else?
                wrap.try_add("3.4.22", krt);
            }
        }
        K::kasun => {
            if dhatu.has_u_in(&["sf\\px~", "u~tfdi~^r"]) {
                wrap.try_add("3.4.17", krt);
            }
        }
        K::ktvA => {
            wrap.try_add("3.4.21", krt);
        }
        _ => (),
    }

    // Exclude this from the `match` above because it contains `kvip`, which is also matched
    // separately above.
    if !wrap.has_krt {
        try_add_krt_for_tacchila_etc(&mut wrap, i, krt);
    }

    Some(wrap.has_krt)
}

/// Runs the rules that add a krt-pratyaya to a given dhatu. Returns whether a pratyaya was added.
pub fn run(p: &mut Prakriya, krt: Krt) -> bool {
    if try_add_upapada_krt(p, krt).unwrap_or(false) {
        return true;
    }
    try_add_krt(p, krt).unwrap_or(false)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::args::{Dhatu, Gana};
    use crate::dhatu_karya;

    fn make_prakriya(dhatu: &str) -> Prakriya {
        let dhatu = Dhatu::new(dhatu, Gana::Bhvadi);
        let mut p = Prakriya::new();
        dhatu_karya::run(&mut p, &dhatu).unwrap();

        p
    }

    fn allows(dhatu: &str, krt: Krt) -> bool {
        let mut p = make_prakriya(dhatu);
        run(&mut p, krt);
        p.terms().last().unwrap().has_u(krt.as_str())
    }

    #[test]
    fn test_common_pratyayas() {
        use Krt::*;

        assert!(allows("BU", tavyat));
        assert!(allows("BU", tavya));
        assert!(allows("BU", anIyar));
        assert!(allows("BU", Rvul));
        assert!(allows("BU", tfc));
        assert!(allows("BU", kta));
        assert!(allows("BU", ktavatu));
        assert!(allows("BU", ktvA));
        assert!(allows("BU", lyuw));
        assert!(allows("BU", tumun));

        assert!(allows("BU", ktin));

        assert!(allows("BU", GaY));
    }

    #[test]
    fn test_ya_pratyaya() {
        use Krt::*;

        assert!(allows("BU", yat));
        // Allowed by "or AvAzyake"
        assert!(allows("BU", Ryat));

        assert!(allows("gada~", yat));
        assert!(!allows("gada~", Ryat));

        assert!(allows("Sa\\kx~", yat));
        assert!(!allows("Sa\\kx~", Ryat));

        assert!(allows("a\\da~", Ryat));
        assert!(!allows("a\\da~", yat));
    }

    #[test]
    fn test_tacchila() {
        use Krt::*;

        // 3.2.161
        assert!(allows("sf\\", kmarac));
        assert!(allows("Gasx~", kmarac));
        assert!(allows("a\\da~", kmarac));
        assert!(!allows("sf\\", tfn));

        // 3.2.162
        assert!(allows("Ba\\njo~", Gurac));
        assert!(allows("BAsf~\\", Gurac));
        assert!(allows("YimidA~\\", Gurac));
        assert!(!allows("Ba\\njo~", tfn));

        // 3.2.173
        assert!(allows("Sf", Aru));
        assert!(allows("vadi~\\", Aru));
        assert!(!allows("Sf", tfn));

        // 3.2.174
        assert!(allows("YiBI\\", kru));
        assert!(allows("YiBI\\", kruka));
        assert!(allows("YiBI\\", klukan));
        assert!(!allows("YiBI\\", tfn));
    }
}
