# 목차

[시작하기 전에](intro.md)

---

# 1권. Rust 문법

## 0부. 읽기 위한 최소 준비

- [0.1 Rust를 왜 이렇게 만들었는가](rust/00-1-why-rust.md)
- [0.2 변수와 `let`, 그리고 기본이 불변인 이유](rust/00-2-variables.md)
- [0.3 타입 표기 읽는 법](rust/00-3-types.md)
- [0.4 구조체, 열거형, 튜플](rust/00-4-data.md)

## 1부. 소유권

- [1.1 소유권이 무엇을 해결하는가](rust/01-1-ownership.md)
- [1.2 이동과 복사](rust/01-2-move.md)
- [1.3 빌림 `&`와 `&mut`](rust/01-3-borrow.md)
- [1.4 `.clone()`이 170번 나오는 이유](rust/01-4-clone.md)
- [1.5 문자열 세 종류](rust/01-5-strings.md)
- [1.6 수명 표기가 왜 여섯 번뿐인가](rust/01-6-lifetimes.md)

## 2부. 부재와 오류

- [2.1 `Option<T>`](rust/02-1-option.md)
- [2.2 `Result<T, E>`](rust/02-2-result.md)
- [2.3 `?` 연산자](rust/02-3-question-mark.md)
- [2.4 `anyhow`와 오류 관례](rust/02-4-anyhow.md)

## 3부. 분기

- [3.1 `match`](rust/03-1-match.md)
- [3.2 `if let`과 `while let`](rust/03-2-if-let.md)
- [3.3 `let ... else`와 `matches!`](rust/03-3-let-else.md)

## 4부. 함수를 값으로 다루기

- [4.1 클로저](rust/04-1-closures.md)
- [4.2 이터레이터](rust/04-2-iterators.md)
- [4.3 `map`, `filter`, `collect` 체인 읽기](rust/04-3-chains.md)

## 5부. 타입에 동작 붙이기

- [5.1 `impl`, 연관 함수와 메서드](rust/05-1-impl.md)
- [5.2 트레이트](rust/05-2-traits.md)
- [5.3 `From`과 `Into`, 그리고 `?`의 나머지 절반](rust/05-3-from-into.md)
- [5.4 `#[derive]`와 serde 속성](rust/05-4-derive.md)

## 6부. 컬렉션

- [6.1 `Vec<T>`와 슬라이스](rust/06-1-vec.md)
- [6.2 `HashMap`과 `HashSet`](rust/06-2-hashmap.md)

## 7부. 코드 조직

- [7.1 모듈과 가시성](rust/07-1-modules.md)
- [7.2 테스트를 같은 파일에 두는 관례](rust/07-2-tests.md)

## 8부. 이 프로젝트 고유의 부분

- [8.1 `macro_rules!` 해부](rust/08-1-macros.md)
- [8.2 채널 `mpsc`로 워처 만들기](rust/08-2-channels.md)
- [8.3 `Arc`와 `async`](rust/08-3-async.md)
- [8.4 다루지 않은 것들](rust/08-4-not-covered.md)

---

# 2권. nunchi 코드 설명

- [0. 지도](nunchi/00-map.md)
- [1. `nunchi index`를 실행하면](nunchi/01-index-command.md)
- [2. 설정을 읽는다](nunchi/02-config.md)
- [3. 파일을 찾는다](nunchi/03-walk.md)
- [4. 코드를 파싱한다](nunchi/04-parse.md)
- [5. 어노테이션을 해석한다](nunchi/05-framework.md)
- [6. SQLite에 저장한다](nunchi/06-store.md)
- [7. 참조를 해소한다](nunchi/07-resolve.md)
- [8. 팩을 만든다](nunchi/08-pack.md)
- [9. 메모리 그래프와 페이지랭크](nunchi/09-graph.md)
- [10. 파일 워처](nunchi/10-watch.md)
- [11. MCP 서버와 TUI](nunchi/11-serve-tui.md)
