//! MyBatis XML 매퍼 (docs/DESIGN.md 4·5절 영속 계층)
//!
//! 한국 기업 Spring 환경에서 MyBatis 비중이 크고, 그중 상당수가 XML 매퍼다.
//! XML은 tree-sitter 문법을 붙이지 않고 직접 훑는다 — 필요한 것이
//! `<select id="...">SQL</select>` 구조뿐이라 정식 파서가 과하다.
//!
//! 매퍼는 `namespace`로 Java 인터페이스와 이어지므로, XML의 statement id가
//! 그 인터페이스의 메서드 이름과 같다. 이 대응이 XML ↔ 코드를 잇는 다리다.

use crate::framework::{tables_in_sql, TableRefFact};
use crate::model::Span;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MapperFacts {
    /// `com.example.OrderMapper` — 대응하는 Java 인터페이스의 FQN
    pub namespace: Option<String>,
    pub statements: Vec<TableRefFact>,
}

const STATEMENT_TAGS: &[(&str, &str)] = &[
    ("select", "select"),
    ("insert", "insert"),
    ("update", "update"),
    ("delete", "delete"),
];

/// 파일이 MyBatis 매퍼인지 값싸게 판별한다.
pub fn looks_like_mapper(source: &str) -> bool {
    source.contains("<mapper") && source.contains("namespace")
}

pub fn parse(source: &str) -> MapperFacts {
    let mut facts = MapperFacts {
        namespace: attribute_after(source, "<mapper", "namespace"),
        statements: Vec::new(),
    };

    for (tag, verb) in STATEMENT_TAGS {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut cursor = 0usize;

        while let Some(rel) = source[cursor..].find(&open) {
            let start = cursor + rel;
            // `<selectKey>` 같은 다른 태그를 `<select`로 오인하지 않는다.
            let next_char = source[start + open.len()..].chars().next();
            if !matches!(next_char, Some(c) if c.is_whitespace() || c == '>') {
                cursor = start + open.len();
                continue;
            }
            let Some(body_start) = source[start..].find('>').map(|i| start + i + 1) else { break };
            let end = source[body_start..]
                .find(&close)
                .map(|i| body_start + i)
                .unwrap_or(source.len());

            let header = &source[start..body_start];
            let id = attribute_after(header, "", "id").unwrap_or_default();
            let sql = &source[body_start..end];
            let line = source[..start].bytes().filter(|b| *b == b'\n').count() as u32 + 1;
            let end_line = source[..end].bytes().filter(|b| *b == b'\n').count() as u32 + 1;

            for (table, sql_verb) in tables_in_sql(sql) {
                facts.statements.push(TableRefFact {
                    owner: id.clone(),
                    table,
                    // 태그가 알려주는 동작을 우선한다 — 본문에 서브쿼리가 섞여도
                    // 이 statement가 무엇을 하는지는 태그가 정확하다.
                    verb: if sql_verb == "select" && *verb != "select" {
                        verb.to_string()
                    } else {
                        sql_verb
                    },
                    span: Span { start_line: line, end_line },
                });
            }
            cursor = end.max(start + open.len());
        }
    }
    facts
}

/// `<mapper namespace="x.Y">` 에서 `x.Y` 를 꺼낸다.
fn attribute_after(source: &str, tag: &str, attr: &str) -> Option<String> {
    let region = if tag.is_empty() {
        source
    } else {
        let idx = source.find(tag)?;
        &source[idx..]
    };
    let key = format!("{attr}=");
    let pos = region.find(&key)?;
    let rest = &region[pos + key.len()..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE mapper PUBLIC "-//mybatis.org//DTD Mapper 3.0//EN" "http://mybatis.org/dtd/mybatis-3-mapper.dtd">
<mapper namespace="com.example.mapper.OrderMapper">

  <select id="findById" resultType="Order">
    SELECT o.*, c.name
      FROM orders o
      JOIN customers c ON c.id = o.customer_id
     WHERE o.id = #{id}
       <if test="status != null"> AND o.status = #{status} </if>
  </select>

  <insert id="save">
    INSERT INTO orders (customer_id, amount) VALUES (#{customerId}, #{amount})
  </insert>

  <delete id="removeById">
    DELETE FROM orders WHERE id = #{id}
  </delete>
</mapper>
"#;

    #[test]
    fn detects_mapper_files() {
        assert!(looks_like_mapper(SAMPLE));
        assert!(!looks_like_mapper("<beans><bean/></beans>"));
    }

    #[test]
    fn extracts_namespace_and_statements() {
        let f = parse(SAMPLE);
        assert_eq!(f.namespace.as_deref(), Some("com.example.mapper.OrderMapper"));

        let ids: Vec<&str> = f.statements.iter().map(|s| s.owner.as_str()).collect();
        assert!(ids.contains(&"findById"), "{ids:?}");
        assert!(ids.contains(&"save"), "{ids:?}");
        assert!(ids.contains(&"removeById"), "{ids:?}");

        // 동적 SQL(<if>)이 섞여도 테이블을 찾아야 한다.
        let find_tables: Vec<&str> = f
            .statements
            .iter()
            .filter(|s| s.owner == "findById")
            .map(|s| s.table.as_str())
            .collect();
        assert!(find_tables.contains(&"orders"), "{find_tables:?}");
        assert!(find_tables.contains(&"customers"), "{find_tables:?}");

        // 태그가 동작을 정한다.
        let del = f.statements.iter().find(|s| s.owner == "removeById").unwrap();
        assert_eq!(del.verb, "delete");
    }

    #[test]
    fn selectkey_is_not_mistaken_for_select() {
        let src = r#"<mapper namespace="X">
  <insert id="save">
    <selectKey keyProperty="id" resultType="long" order="AFTER">
      SELECT LAST_INSERT_ID()
    </selectKey>
    INSERT INTO orders (name) VALUES (#{name})
  </insert>
</mapper>"#;
        let f = parse(src);
        let owners: Vec<&str> = f.statements.iter().map(|s| s.owner.as_str()).collect();
        assert_eq!(owners, vec!["save"], "selectKey를 statement로 세면 안 된다");
    }
}
