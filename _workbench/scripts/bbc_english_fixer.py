#!/usr/bin/env python3
"""
BBC English Fixer for ReedCMS
Converts American English to BBC English in Rust comments and docstrings ONLY.

CRITICAL: Only modifies content in:
- Line comments starting with //
- Doc comments starting with ///
- Block comments /* ... */

Does NOT modify:
- Rust code (variable names, function names, etc.)
- String literals in code
- Dependency names (actix_web, serde, etc.)
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple

# BBC English conversion rules
# Format: (american_pattern, british_replacement, description)
CONVERSION_RULES: List[Tuple[str, str, str]] = [
    # -ize/-ization → -ise/-isation
    (r"\bauthorization\b", "authorisation", "-ize to -ise"),
    (r"\bAuthorization\b", "Authorisation", "-ize to -ise"),
    (r"\bunauthorized\b", "unauthorised", "-ize to -ise"),
    (r"\bUnauthorized\b", "Unauthorised", "-ize to -ise"),
    (r"\borganization\b", "organisation", "-ize to -ise"),
    (r"\bOrganization\b", "Organisation", "-ize to -ise"),
    (r"\boptimization\b", "optimisation", "-ize to -ise"),
    (r"\bOptimization\b", "Optimisation", "-ize to -ise"),
    (r"\boptimize\b", "optimise", "-ize to -ise"),
    (r"\bOptimize\b", "Optimise", "-ize to -ise"),
    (r"\boptimized\b", "optimised", "-ize to -ise"),
    (r"\bOptimized\b", "Optimised", "-ize to -ise"),
    (r"\boptimizing\b", "optimising", "-ize to -ise"),
    (r"\bOptimizing\b", "Optimising", "-ize to -ise"),
    (r"\binitialize\b", "initialise", "-ize to -ise"),
    (r"\bInitialize\b", "Initialise", "-ize to -ise"),
    (r"\binitialized\b", "initialised", "-ize to -ise"),
    (r"\bInitialized\b", "Initialised", "-ize to -ise"),
    (r"\binitialization\b", "initialisation", "-ize to -ise"),
    (r"\bInitialization\b", "Initialisation", "-ize to -ise"),
    (r"\bcentralize\b", "centralise", "-ize to -ise"),
    (r"\bCentralize\b", "Centralise", "-ize to -ise"),
    (r"\brecognize\b", "recognise", "-ize to -ise"),
    (r"\bRecognize\b", "Recognise", "-ize to -ise"),
    (r"\banalyze\b", "analyse", "-yze to -yse"),
    (r"\bAnalyze\b", "Analyse", "-yze to -yse"),
    (r"\banalyzed\b", "analysed", "-yze to -yse"),
    (r"\bAnalyzed\b", "Analysed", "-yze to -yse"),
    (r"\banalyzing\b", "analysing", "-yze to -yse"),
    (r"\bAnalyzing\b", "Analysing", "-yze to -yse"),
    # -or → -our (CAREFUL: not in code identifiers!)
    (r"\bcolor\b", "colour", "-or to -our"),
    (r"\bColor\b", "Colour", "-or to -our"),
    (r"\bhonor\b", "honour", "-or to -our"),
    (r"\bHonor\b", "Honour", "-or to -our"),
    (r"\bbehavior\b", "behaviour", "-or to -our"),
    (r"\bBehavior\b", "Behaviour", "-or to -our"),
    (r"\bfavor\b", "favour", "-or to -our"),
    (r"\bFavor\b", "Favour", "-or to -our"),
    # -er → -re
    (r"\bcenter\b", "centre", "-er to -re"),
    (r"\bCenter\b", "Centre", "-er to -re"),
    (r"\bmeter\b", "metre", "-er to -re"),
    (r"\bMeter\b", "Metre", "-er to -re"),
    # -ense → -ence
    (r"\bdefense\b", "defence", "-ense to -ence"),
    (r"\bDefense\b", "Defence", "-ense to -ence"),
    (r"\boffense\b", "offence", "-ense to -ence"),
    (r"\bOffense\b", "Offence", "-ense to -ence"),
    (r"\blicense\b(?!\s*=)", "licence", '-ense to -ence (not in "license = " cargo)'),
    (r"\bLicense\b", "Licence", "-ense to -ence"),
    # -og → -ogue
    (r"\bdialog\b", "dialogue", "-og to -ogue"),
    (r"\bDialog\b", "Dialogue", "-og to -ogue"),
    (r"\bcatalog\b", "catalogue", "-og to -ogue"),
    (r"\bCatalog\b", "Catalogue", "-og to -ogue"),
    # Single → double L
    (r"\btraveled\b", "travelled", "single to double L"),
    (r"\bTraveled\b", "Travelled", "single to double L"),
    (r"\btraveling\b", "travelling", "single to double L"),
    (r"\bTraveling\b", "Travelling", "single to double L"),
    (r"\bmodeling\b", "modelling", "single to double L"),
    (r"\bModeling\b", "Modelling", "single to double L"),
    # ae/oe
    (r"\bpediatric\b", "paediatric", "ae/oe"),
    (r"\bPediatric\b", "Paediatric", "ae/oe"),
    (r"\bmaneuver\b", "manoeuvre", "ae/oe"),
    (r"\bManeuver\b", "Manoeuvre", "ae/oe"),
]

# Patterns to identify comment sections
COMMENT_PATTERNS = [
    # Line comments: // or ///
    (r"(^\s*//[/!]?\s*)(.*)$", 1, 2),  # group 1 = prefix, group 2 = content
]


def is_rust_keyword(word: str) -> bool:
    """Check if word is a Rust keyword or common identifier."""
    keywords = {
        "impl",
        "mod",
        "use",
        "pub",
        "fn",
        "let",
        "mut",
        "const",
        "static",
        "struct",
        "enum",
        "trait",
        "type",
        "match",
        "if",
        "else",
        "while",
        "for",
        "loop",
        "break",
        "continue",
        "return",
        "async",
        "await",
        # Common crates/modules - DO NOT CHANGE
        "serde",
        "actix_web",
        "actix_rt",
        "tokio",
        "num_cpus",
        "serialize",
        "deserialize",  # serde traits
    }
    return word.lower() in keywords


def is_code_example_line(line: str) -> bool:
    """
    Check if this comment line is part of a code example.
    Returns True for lines like:
    - /// ```rust
    - /// let color = ...
    - /// ```
    """
    # Check for code fence markers
    if "```" in line:
        return True

    # Check for common code patterns in comments
    # (lines with assignment, function calls, type annotations)
    code_patterns = [
        r"//.*\blet\s+\w+\s*=",  # let x =
        r"//.*\bfn\s+\w+\s*\(",  # fn foo(
        r"//.*\w+\s*\(",  # function_call(
        r"//.*\w+::\w+",  # Type::method
        r"//.*\.\w+\(",  # .method(
        r"//.*\s*\|\s*\w+\s*\|",  # |x| closure
    ]

    for pattern in code_patterns:
        if re.search(pattern, line):
            return True

    return False


def fix_line(
    line: str,
    line_num: int,
    filename: str,
    dry_run: bool = True,
    in_code_block: bool = False,
) -> Tuple[str, List[str], bool]:
    """
    Fix BBC English in a single line if it's a comment.
    Returns: (fixed_line, list_of_changes, is_in_code_block)
    """
    changes = []

    # Track if we're entering/exiting a code block
    if "```" in line:
        in_code_block = not in_code_block
        return line, changes, in_code_block

    # Skip if we're inside a code block or this looks like a code example
    if in_code_block or is_code_example_line(line):
        return line, changes, in_code_block

    # Check if this is a comment line
    is_comment = False
    for pattern, prefix_group, content_group in COMMENT_PATTERNS:
        match = re.match(pattern, line)
        if match:
            is_comment = True
            prefix = match.group(prefix_group)
            content = match.group(content_group)
            original_content = content

            # Apply all conversion rules to the comment content only
            for american, british, description in CONVERSION_RULES:
                if re.search(american, content):
                    content = re.sub(american, british, content)
                    changes.append(
                        f"{filename}:{line_num}: {description} | "
                        f"'{re.search(american, original_content).group()}' → '{british}'"
                    )

            if content != original_content:
                return prefix + content + "\n", changes, in_code_block
            break

    return line, changes, in_code_block


def process_file(filepath: Path, dry_run: bool = True) -> Tuple[int, List[str]]:
    """
    Process a single Rust file.
    Returns: (number_of_changes, list_of_change_descriptions)
    """
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            lines = f.readlines()
    except Exception as e:
        print(f"ERROR reading {filepath}: {e}", file=sys.stderr)
        return 0, []

    new_lines = []
    all_changes = []
    in_code_block = False

    for line_num, line in enumerate(lines, 1):
        fixed_line, changes, in_code_block = fix_line(
            line, line_num, str(filepath), dry_run, in_code_block
        )
        new_lines.append(fixed_line)
        all_changes.extend(changes)

    # Write back if not dry run and changes were made
    if not dry_run and all_changes:
        try:
            with open(filepath, "w", encoding="utf-8") as f:
                f.writelines(new_lines)
        except Exception as e:
            print(f"ERROR writing {filepath}: {e}", file=sys.stderr)
            return 0, []

    return len(all_changes), all_changes


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Convert American English to BBC English in Rust comments"
    )
    parser.add_argument("path", type=Path, help="File or directory to process")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=True,
        help="Show what would be changed without modifying files (default: True)",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Actually apply the changes (overrides --dry-run)",
    )
    parser.add_argument("--verbose", action="store_true", help="Show all changes")

    args = parser.parse_args()

    # Determine dry_run mode
    dry_run = not args.apply

    if dry_run:
        print("🔍 DRY RUN MODE - No files will be modified")
        print("   Use --apply to actually make changes\n")
    else:
        print("⚠️  APPLY MODE - Files WILL be modified!\n")

    # Collect all .rs files
    rust_files = []
    if args.path.is_file():
        if args.path.suffix == ".rs":
            rust_files.append(args.path)
    elif args.path.is_dir():
        rust_files = list(args.path.rglob("*.rs"))
    else:
        print(f"ERROR: {args.path} is not a file or directory", file=sys.stderr)
        sys.exit(1)

    print(f"Processing {len(rust_files)} Rust files...\n")

    total_changes = 0
    files_with_changes = 0

    for rust_file in sorted(rust_files):
        num_changes, changes = process_file(rust_file, dry_run)

        if num_changes > 0:
            files_with_changes += 1
            total_changes += num_changes

            print(
                f"{'[DRY RUN] ' if dry_run else ''}📝 {rust_file.relative_to(args.path.parent if args.path.is_file() else args.path)}"
            )
            print(
                f"   {num_changes} change(s) {'would be made' if dry_run else 'made'}"
            )

            if args.verbose:
                for change in changes:
                    print(f"   • {change}")
            print()

    print("=" * 80)
    print(f"Summary:")
    print(f"  Files processed: {len(rust_files)}")
    print(f"  Files with changes: {files_with_changes}")
    print(f"  Total changes: {total_changes}")

    if dry_run and total_changes > 0:
        print(f"\n💡 Run with --apply to make these changes")

    sys.exit(0 if total_changes == 0 else 1)


if __name__ == "__main__":
    main()
