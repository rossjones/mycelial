pub mod destination;

use arrow::{
    array::{downcast_array, Array, AsArray, PrimitiveArray, StringArray},
    datatypes::{
        DataType, Float16Type, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
    },
    record_batch::RecordBatch,
};
use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use section::Message as _Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type Message = _Message<BacalhauPayload>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacalhauPayload {
    pub data: HashMap<String, String>,
}

impl BacalhauPayload {
    fn new(pairs: impl Iterator<Item = (String, String)>) -> Self {
        Self {
            data: pairs.into_iter().collect(),
        }
    }
}

impl TryFrom<RecordBatch> for BacalhauPayload {
    type Error = StdError;

    fn try_from(batch: RecordBatch) -> Result<Self, Self::Error> {
        Self::try_from(&batch)
    }
}

fn first_item<T>(array: &dyn Array) -> T::Native
where
    T: arrow::array::ArrowPrimitiveType,
{
    downcast_array::<PrimitiveArray<T>>(array)
        .into_iter()
        .nth(0)
        .unwrap()
        .unwrap()
}

impl TryFrom<&RecordBatch> for BacalhauPayload {
    type Error = StdError;

    fn try_from(batch: &RecordBatch) -> Result<Self, Self::Error> {
        // Takes the provided RecordBatch and processes the first item
        // (only) by proving the payload constructor a vec of k, v tuples
        let sub = batch.slice(0, 1);
        let schema = sub.schema();

        let names = schema.fields().iter().map(|f| f.name().to_string());
        let values: Vec<_> = sub
            .columns()
            .iter()
            .map(|column| {
                let array = column.as_ref();
                // FIXME: is it possible to use downcast_macro from arrow?
                match array.data_type() {
                    DataType::Int8 => first_item::<Int8Type>(array).to_string(),
                    DataType::Int16 => first_item::<Int16Type>(array).to_string(),
                    DataType::Int32 => first_item::<Int32Type>(array).to_string(),
                    DataType::Int64 => first_item::<Int64Type>(array).to_string(),
                    DataType::Float16 => first_item::<Float16Type>(array).to_string(),
                    DataType::Float32 => first_item::<Float32Type>(array).to_string(),
                    DataType::Float64 => first_item::<Float64Type>(array).to_string(),
                    DataType::Binary => {
                        let blob = array.as_binary::<i32>().iter().nth(0).unwrap().unwrap();
                        BASE64.encode(blob)
                    }
                    DataType::Utf8 => downcast_array::<StringArray>(array)
                        .into_iter()
                        .nth(0)
                        .unwrap_or(Some(""))
                        .unwrap()
                        .to_string(),

                    t => unimplemented!("unimplemented {}", t),
                }
            })
            .collect();

        Ok(BacalhauPayload::new(names.zip(values)))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::{
        array::{BinaryArray, Int32Array},
        datatypes::{Field, Schema},
    };

    use super::*;

    #[test]
    fn test_batch_to_payload() -> Result<(), StdError> {
        let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
        let schema = Schema::new(vec![Field::new("id", DataType::Int32, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(id_array)]).unwrap();
        let payload: BacalhauPayload = batch.try_into()?;
        assert_eq!(payload.data.get("id").unwrap(), &"1".to_string());

        let name_array = StringArray::from(vec!["one"]);
        let schema = Schema::new(vec![Field::new("name", DataType::Utf8, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(name_array)]).unwrap();
        let payload: BacalhauPayload = batch.try_into()?;
        assert_eq!(payload.data.get("name").unwrap(), &"one".to_string());

        let name_array = StringArray::from(vec!["one"]);
        let id_array = Int32Array::from(vec![1]);
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]);
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(id_array), Arc::new(name_array)],
        )
        .unwrap();
        let payload: BacalhauPayload = batch.try_into()?;
        assert_eq!(payload.data.get("id").unwrap(), &"1".to_string());
        assert_eq!(payload.data.get("name").unwrap(), &"one".to_string());

        let values: Vec<&[u8]> = vec![b"one", b"two", b"", b"three"];
        let array = BinaryArray::from_vec(values);
        let schema = Schema::new(vec![Field::new("v", DataType::Binary, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(array)]).unwrap();
        let payload: BacalhauPayload = batch.try_into()?;
        // We expect 'one' to be base64 encoded ..
        assert_eq!(payload.data.get("v").unwrap(), &String::from("b25l"));

        Ok(())
    }
}
