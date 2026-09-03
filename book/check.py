#!/usr/bin/env python3
"""책 문서의 규칙을 검사한다.

사람이 눈으로 지키기 어려운 세 가지를 확인한다.
  1. 선행 링크가 뒤쪽 장을 가리키지 않는가
  2. {{#include}}가 가리키는 앵커가 소스에 실제로 있는가
  3. 연습문제가 정말로 실패하는 상태인가

사용법: python3 book/check.py
"""
import re
import subprocess
import sys
from pathlib import Path

BOOK = Path(__file__).parent
SRC = BOOK / "src"
REPO = BOOK.parent
EX = BOOK / "exercises"

errors: list[str] = []
warnings: list[str] = []


def chapter_order() -> dict[str, int]:
    """SUMMARY.md에 적힌 순서대로 장에 번호를 매긴다."""
    summary = (SRC / "SUMMARY.md").read_text(encoding="utf-8")
    order = {}
    for i, m in enumerate(re.finditer(r"\]\(([^)]+\.md)\)", summary)):
        order[m.group(1)] = i
    return order


def check_link_direction(order: dict[str, int]) -> None:
    """선행 링크가 앞쪽을 가리키는지 확인한다.

    문서 안에서 '선행 장'으로 표시한 링크가 자기보다 뒤에 오는 장을 가리키면
    선행 규칙이 깨진 것이다. 읽는 사람이 아직 모르는 개념을 전제하게 된다.
    """
    for md in sorted(SRC.rglob("*.md")):
        rel = str(md.relative_to(SRC))
        if rel == "SUMMARY.md" or rel not in order:
            continue
        me = order[rel]
        text = md.read_text(encoding="utf-8")
        # '> **선행 장**:' 줄에 있는 링크만 검사한다.
        for line in text.splitlines():
            if "선행 장" not in line:
                continue
            for target in re.findall(r"\]\(([^)]+\.md)\)", line):
                resolved = str((md.parent / target).resolve().relative_to(SRC.resolve()))
                if resolved not in order:
                    errors.append(f"{rel}: 선행 링크 대상이 목차에 없다 → {target}")
                elif order[resolved] >= me:
                    errors.append(
                        f"{rel}: 선행 장이 뒤에 있다 → {target} "
                        f"(이 장 {me}번, 대상 {order[resolved]}번)"
                    )


def check_anchors() -> None:
    """{{#include ...:anchor}}가 가리키는 앵커가 소스에 있는지 확인한다."""
    pattern = re.compile(r"\{\{#include\s+([^:}]+):([A-Za-z0-9_]+)\s*\}\}")
    for md in sorted(SRC.rglob("*.md")):
        text = md.read_text(encoding="utf-8")
        for path, anchor in pattern.findall(text):
            target = (md.parent / path.strip()).resolve()
            rel = md.relative_to(SRC)
            if not target.is_file():
                errors.append(f"{rel}: 인용 대상 파일이 없다 → {path}")
                continue
            body = target.read_text(encoding="utf-8")
            if f"ANCHOR: {anchor}" not in body:
                errors.append(f"{rel}: 앵커가 소스에 없다 → {path}:{anchor}")
            elif f"ANCHOR_END: {anchor}" not in body:
                errors.append(f"{rel}: 앵커 끝 표시가 없다 → {path}:{anchor}")


def check_exercises_fail() -> None:
    """연습문제가 정말로 실패하는 상태인지 확인한다.

    문제를 만들어 놓고 실제로는 이미 통과하는 상태였다는 실수를 막는다.
    그런 문제는 풀 것이 없으므로 문제가 아니다.
    """
    if not EX.is_dir():
        return
    crates = sorted(p.name for p in EX.iterdir() if p.name.startswith("ex_"))
    if not crates:
        return
    for name in crates:
        proc = subprocess.run(
            ["cargo", "test", "-p", name, "--quiet"],
            cwd=EX,
            capture_output=True,
            text=True,
        )
        if proc.returncode == 0:
            errors.append(f"연습문제 {name}: 이미 통과한다. 풀 것이 없으므로 문제가 아니다")


def check_solutions_exist() -> None:
    """모든 문제에 정답 파일이 있는지 확인한다."""
    if not EX.is_dir():
        return
    for p in sorted(EX.iterdir()):
        if not p.name.startswith("ex_"):
            continue
        if not (EX / "solutions" / f"{p.name}.rs").is_file():
            errors.append(f"연습문제 {p.name}: 정답 파일이 없다")


def check_summary_files_exist(order: dict[str, int]) -> None:
    """목차에 있는 장이 실제로 쓰였는지 확인한다.

    mdbook build 는 목차에 있으나 없는 파일을 빈 파일로 만들어 둔다.
    그래서 "파일이 있는가" 만 보면 아직 쓰지 않은 장을 놓친다.
    내용이 있는지까지 확인한다.
    """
    for rel in order:
        path = SRC / rel
        if not path.is_file():
            warnings.append(f"목차에 있으나 파일이 없다 → {rel}")
        elif len(path.read_text(encoding="utf-8").strip()) < 100:
            warnings.append(f"아직 쓰지 않은 장 → {rel}")


def main() -> int:
    order = chapter_order()
    check_summary_files_exist(order)
    check_link_direction(order)
    check_anchors()
    check_solutions_exist()
    if "--full" in sys.argv:
        check_exercises_fail()

    for w in warnings:
        print(f"경고: {w}")
    for e in errors:
        print(f"오류: {e}")

    print(f"\n장 {len(order)}개 · 경고 {len(warnings)}건 · 오류 {len(errors)}건")
    if errors:
        print("\n검사에 실패했다.")
        return 1
    print("검사를 통과했다.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
