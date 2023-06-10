import concurrent.futures
import subprocess


def pascal_case(var: str) -> str:
    return "".join(word.capitalize() for word in var.split('-'))


def enum_arm(icon: str) -> str:
    return f"""\t/// {icon}\n\t{pascal_case(icon)},\n"""


def match_arm(icon: str, i: int) -> str:
    unicode = hex(0x61 + i)[2:]
    # unicode = hex(0xf102 + i)[2:]
    return f"""\t\tIcon::{pascal_case(icon)} => '\\u{{{unicode}}}',\n"""


with open('list.txt') as f:
    icons = [line.strip() for line in f.readlines()]
    svg_path = "/mnt/c/Users/andre/Downloads/bootstrap-icons-1.10.5/bootstrap-icons-1.10.5"

    print('1. svg stroke to path')
    stroke_to_path_processes = [
        subprocess.Popen([
            "/usr/local/bin/svg-stroke-to-path",
            "SameStrokeColor",
            'stroke="#000"',
            f'{svg_path}/{icon}.svg'
        ]) for icon in icons
    ]
    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = [executor.submit(process.wait) for process in stroke_to_path_processes]
        concurrent.futures.wait(futures)

    print('2. import into FontForge')
    command = icons.copy()
    command.insert(0, "bash")
    command.insert(1, "run_fontforge.sh")
    subprocess.run(command)

    print('3. rust file')
    enum_arms = "".join(enum_arm(icon) for icon in icons)
    match_arms = "".join(match_arm(icon, i) for i, icon in enumerate(icons))

    rust = f"""//! Selected boostrap icons. Machine generated code. Do not change!

/// Icons
#[derive(Copy, Clone, Debug, Hash)]
pub enum Icon {{
{enum_arms}}}

/// Converts an icon into a char.
#[must_use]
#[allow(clippy::too_many_lines)]
pub const fn icon_to_char(icon: Icon) -> char {{
    match icon {{
{match_arms}\t}}
}}

impl std::fmt::Display for Icon {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        use std::fmt::Write;
        f.write_char(icon_to_char(*self))
    }}
}}
"""
    with open("../../src/icon.rs", "w") as out_f:
        out_f.write(rust)
