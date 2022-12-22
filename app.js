'use strict';

function spells(search, f) {
    fetch("resources/spells.json")
        .then(response => response.json())
        .catch(e => console.log(e))
        .then(spells => spells.filter(spell =>
            (search.needle ? spell.name.toLowerCase().includes(search.needle) : true)
            && (search.levels ? search.levels[spell.level] : true)
            && (search.classes ? spell.classes.some(clss => search.classes.includes(clss)) : true)
            && (search.schools ? search.schools.includes(spell.school) : true)))
        .then(f)
}

function displaySpell(spell) {
    function listGramatically(arr) {
        if (arr.length === 0) {
            return "";
        }
        const last = arr.length - 1;
        let ret = "";
        for (const j in arr) {
            const i = parseInt(j)
            if (i !== 0) {
                ret += i === last
                    ? (i === 1
                        ? " and "
                        : ", and ")
                    : ", ";
            }
            ret += arr[i];
        }
        return ret
    }

    const higherLevels = spell.higher_levels
        ? `
            <hr>
            <h3>At higher levels:</h3>
            <p>${spell.higher_levels}</p>
        `
        : "";

    const a_an = spell.classes[0] === "Artificer" ? "An" : "A";

    return `
<!--        <div class="spells">-->
            <h1 class="spell_part">${spell.name}</h1>
            <hr>
            <p>${spell.school}</p>
            <p>Level: ${spell.level === 0 ? "Cantrip" : spell.level}</p>
            <p>Casting time: ${spell.casting_time}</p>
            <p>Range: ${spell.range}</p>
            <p>Components: ${spell.components}</p>
            <p>Duration: ${spell.duration}</p>
            <p>Ritual: ${spell.ritual ? "Yes" : "No"}</p>
            <hr>
            <p>${spell.description}</p>
            ${higherLevels}
            <hr>
            <p>${a_an} ${listGramatically(spell.classes)} spell from ${spell.source} page ${spell.page}.</p>
            <p></p>
<!--        </div>-->
    `
}

function setSpells() {
    const levels = Array.from({length: 10}, (_, i) => document.getElementById(`level-${i}`).classList.contains("btn-active"));
    const classes = [
        "Artificer",
        "Bard",
        "Cleric",
        "Druid",
        "Paladin",
        "Ranger",
        "Sorcerer",
        "Warlock",
        "Wizard"
    ].filter(clss => document.getElementById(clss).classList.contains("btn-active"));
    const schools = [
        "Abjuration",
        "Conjuration",
        "Divination",
        "Enchantment",
        "Evocation",
        "Illusion",
        "Transmutation",
        "Necromancy"
    ].filter(school => document.getElementById(school).classList.contains("btn-active"));
    // console.log(classes.length)
    const search = {
        needle: document.getElementById("spellName").value.toLowerCase(),
        levels: levels.every(b => !b) ? null : levels,
        classes: classes.length === 0 ? null : classes,
        schools: schools.length === 0 ? null : schools,
    };
    // console.log(search.levels);
    spells(search, spells => {
        spells.sort((a, b) => {
            if (a.name < b.name) return -1;
            if (a.name > b.name) return 1;
            return 0;
        });
        const html = spells.map(displaySpell).join("\n");
        document.getElementById("spells").innerHTML = html
    })
}

function spellName() {
    setSpells()
}

function loadSpells() {
    setSpells()
}

function levelSelect(level) {
    const button = document.getElementById(`level-${level}`);
    button.classList.toggle("btn-active")
    button.classList.toggle("btn-inactive")
    setSpells()
    // document.getElementById()
}


function classSelect(clss) {
    const button = document.getElementById(clss);
    button.classList.toggle("btn-active")
    button.classList.toggle("btn-inactive")
    setSpells()
}

function schoolSelect(school) {
    const button = document.getElementById(school);
    button.classList.toggle("btn-active")
    button.classList.toggle("btn-inactive")
    setSpells()
}
