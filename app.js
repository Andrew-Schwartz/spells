'use strict';

function spells(needle, f) {
    fetch("resources/spells.json")
        .then(response => response.json())
        .catch(e => console.log(e))
        .then(spells => spells.filter(spell => spell.name.toLowerCase().includes(needle)))
        .then(f)
}

function displaySpell(spell) {
    const higherLevels = spell.higher_levels
        ? `
            <hr>
            <h3>At higher levels:</h3>
            <p>${spell.higher_levels}</p>
        `
        : "";
    return `
        <div class="spell">
            <h1 class="spell_part">${spell.name}</h1>
            <hr>
            <p>${spell.school}</p>
            <p>Level: ${spell.level}</p>
            <p>Casting time: ${spell.casting_time}</p>
            <p>Range: ${spell.range}</p>
            <p>Components: ${spell.components}</p>
            <p>Duration: ${spell.duration}</p>
            <p>Ritual: ${spell.ritual ? "yes" : "no"}</p>
            <hr>
            <p>${spell.description}</p>
            ${higherLevels}
            <p></p>
        </div>
    `
}

function setSpells(needle) {
    spells(needle, spells => {
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
    const name = document.getElementById("spellName").value;
    // document.getElementById("sn").innerHTML = "Spell: " + name;
    setSpells(name)
    // spells(input, names => {
    //     document.getElementById("load").innerHTML = names
    // })
}

function loadSpells() {
    // alert(spells)
    // const name = document.getElementById("spellName").value;
    setSpells("")
}