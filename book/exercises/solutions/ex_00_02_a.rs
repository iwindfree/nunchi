// 정답: `let mut total = 0;`
//
// 값을 바꾸려면 `mut`을 붙여야 합니다. Rust에서 변수는 기본적으로
// 바꿀 수 없습니다.

pub fn sum_lines(counts: &[u32]) -> u32 {
    let mut total = 0;
    for c in counts {
        total = total + c;
    }
    total
}

// 참고: 실제로는 이터레이터를 쓰는 편이 관례에 맞습니다(4부에서 다룹니다).
//
//     counts.iter().sum()
