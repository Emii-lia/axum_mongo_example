use mongodb::bson::{doc, Document};

pub fn populate(
  from: &str,
  local_field: &str,
  foreign_field: &str,
  as_field: &str
) -> Vec<Document> {
  vec![
    doc! {
      "$lookup": {
        "from": from,
        "localField": local_field,
        "foreignField": foreign_field,
        "as": as_field
      }
    },
    doc! {
      "$unwind": {
        "path": format!("${}", as_field),
        "preserveNullAndEmptyArrays": true
      }
    }
  ]
}

pub fn project(fields: Vec<(&str, i32)>) -> Document {
  let mut projection = Document::new();
  for (field, include) in fields {
    projection.insert(field, include);
  }
  doc! { "$project": projection }
}

pub fn match_filter(filter: Document) -> Document {
  doc! { "$match": filter }
}