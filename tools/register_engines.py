#!/usr/bin/env python3
"""Register engine files into src/engines/mod.rs and settings.toml (idempotent).

Older engines take their id from the existing build() dispatch arm; newer files
carry a "// engine-id: <id>" comment. Both are supported.
"""
import glob
import os
from pathlib import Path
import re
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MOD = os.path.join(ROOT, "src", "engines", "mod.rs")
SETTINGS = os.path.join(ROOT, "settings.toml")
ENGINES_DIR = os.path.join(ROOT, "src", "engines")


def module_name(filename):
    name = filename[:-3]
    return "r" + name if name[0].isdigit() else name


def filter_engine_settings(settings: str, valid_engine_ids: set[str]) -> str:
    """Keep implemented engine entries without depending on TOML quoting or spacing.

    Raises:
        ValueError: An engine entry omits a non-empty string engine type.
        tomllib.TOMLDecodeError: The settings are not valid TOML.
    """
    entries = tomllib.loads(settings).get("engines", [])
    sections = re.split(r"(?m)(?=^[ \t]*\[\[engines\]\][ \t]*(?:#.*)?$)", settings)
    kept = [sections[0]]
    for entry, section in zip(entries, sections[1:], strict=True):
        engine_id = entry.get("engine")
        if not isinstance(engine_id, str) or not engine_id:
            raise ValueError(
                f"engine registration: entry {entry.get('name', '<unnamed>')!r} "
                "needs a non-empty string engine type"
            )
        if engine_id in valid_engine_ids:
            kept.append(section)
    return "".join(kept)


def main():
    # module -> engine id, taken from the dispatch arms that exist today
    mod = Path(MOD).read_text(encoding="utf-8")
    existing_arm = {
        m.group(2): m.group(1)
        for m in re.finditer(
            r'^        "([^"]+)" => (\w+)::\w+::from_config\(config\)\?,', mod, flags=re.M
        )
    }
    declared = set(re.findall(r"^pub mod (\w+);", mod, flags=re.M))

    engines = []
    for path in sorted(glob.glob(os.path.join(ENGINES_DIR, "*.rs"))):
        base = os.path.basename(path)
        if base == "mod.rs":
            continue
        module = module_name(base)
        text = Path(path).read_text(encoding="utf-8")
        if "pub fn from_config" not in text or "impl Engine for" not in text:
            print("[skip incomplete]", base)
            continue
        m_id = re.search(r"// engine-id:\s*(\S+)", text)
        engine_id = m_id.group(1) if m_id else existing_arm.get(module)
        m_struct = re.search(r"pub struct (\w+)", text)
        m_short = re.search(r'shortcut: "([^"]*)"', text)
        m_cats = re.search(r"categories: &\[([^\]]*)\]", text)
        engines.append(
            {
                "module": module,
                "engine_id": engine_id,
                "struct": m_struct.group(1) if m_struct else None,
                "shortcut": m_short.group(1) if m_short else "",
                "categories": re.findall(r'"([^"]+)"', m_cats.group(1)) if m_cats else ["general"],
                "paging": "paging: true" in text,
                "time_range": "time_range_support: true" in text,
                "safesearch": "safesearch: true" in text,
            }
        )

    with_id = [e for e in engines if e["engine_id"] and e["struct"]]
    no_id = [e["module"] for e in engines if not (e["engine_id"] and e["struct"])]
    if no_id:
        print("[no engine id, skipped]", ", ".join(no_id))

    # Remove registrations whose implementation was deleted before adding engines.
    valid = {e["module"] for e in with_id}
    mod = re.sub(
        r"(?m)^pub mod (\w+);\n",
        lambda match: match.group(0) if match.group(1) in valid else "",
        mod,
    )

    # 1. declarations: keep the existing block, append missing modules
    add = [e["module"] for e in with_id if e["module"] not in declared]
    if add:
        block = "\n".join("pub mod " + module + ";" for module in add) + "\n"
        mod = re.sub(
            r"(?m)(^pub mod \w+;\n(?:pub mod \w+;\n)*)", lambda m: m.group(1) + block, mod, count=1
        )

    # 2. dispatch arms: drop arms whose module file is gone, add missing new ones
    kept = []
    for line in mod.split("\n"):
        m = re.match(r'^(\s+)"[^"]+" => \w+::\w+::from_config\(config\)\?,\s*$', line)
        if m and re.search(r"=> (\w+)::", line).group(1) not in valid:
            continue
        kept.append(line)
    mod = "\n".join(kept)

    present = set(re.findall(r'^        "([^"]+)" =>', mod, flags=re.M))
    new_arms = "".join(
        '        "' + e["engine_id"] + '" => ' + e["module"] + "::" + e["struct"]
        + "::from_config(config)?,\n"
        for e in with_id
        if e["engine_id"] not in present
    )
    if new_arms:
        mod = mod.replace("        other => {", new_arms + "        other => {", 1)
    # 3. settings.toml entries
    settings = Path(SETTINGS).read_text(encoding="utf-8")
    valid_engine_ids = {e["engine_id"] for e in with_id}
    settings = filter_engine_settings(settings, valid_engine_ids)
    entries = tomllib.loads(settings).get("engines", [])
    have_names = {entry.get("name") for entry in entries}
    have_shortcuts = {entry.get("shortcut") for entry in entries}
    have_engine = {entry["engine"] for entry in entries}
    blocks = []
    for e in with_id:
        if e["engine_id"] in have_engine or e["module"] in have_names:
            continue
        lines = ["[[engines]]", 'name = "' + e["module"] + '"', 'engine = "' + e["engine_id"] + '"']
        if e["shortcut"] and e["shortcut"] not in have_shortcuts:
            have_shortcuts.add(e["shortcut"])
            lines.append('shortcut = "' + e["shortcut"] + '"')
        lines.append("categories = [" + ", ".join('"' + c + '"' for c in e["categories"]) + "]")
        if e["paging"]:
            lines.append("paging = true")
        if e["time_range"]:
            lines.append("time_range_support = true")
        lines.append("disabled = true  # unverified; remove when confirmed working")
        blocks.append("\n".join(lines))
        have_engine.add(e["engine_id"])
        have_names.add(e["module"])
    if blocks:
        settings = settings.rstrip("\n") + "\n\n" + "\n\n".join(blocks) + "\n"
    # Validate all settings before mutating either generated file.
    with open(MOD, "w", encoding="utf-8") as handle:
        handle.write(mod)
    with open(SETTINGS, "w", encoding="utf-8") as handle:
        handle.write(settings)
    print("engine modules:", len(with_id), "| new modules:", len(add))
    print("settings entries added:", len(blocks))


if __name__ == "__main__":
    main()
