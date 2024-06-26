use crate::errors::ConnectorXPythonError;
use arrow2::{
    array::ArrayRef,
    chunk::Chunk,
    datatypes::{Field, Schema},
    ffi,
};
use connectorx::source_router::SourceConn;
use connectorx::{prelude::*, sql::CXQuery};
use fehler::throws;
use libc::uintptr_t;
use pyo3::prelude::*;
use pyo3::{PyAny, Python};
use std::sync::Arc;

#[throws(ConnectorXPythonError)]
pub fn write_arrow<'a>(
    py: Python<'a>,
    source_conn: &SourceConn,
    origin_query: Option<String>,
    queries: &[CXQuery<String>],
) -> &'a PyAny {
    let destination = get_arrow2(source_conn, origin_query, queries)?;
    let (rbs, schema) = destination.arrow()?;
    let ptrs = to_ptrs(rbs, schema);
    let obj: PyObject = ptrs.into_py(py);
    obj.into_ref(py)
}

fn to_ptrs(
    rbs: Vec<Chunk<ArrayRef>>,
    schema: Arc<Schema>,
) -> (Vec<String>, Vec<Vec<(uintptr_t, uintptr_t)>>) {
    if rbs.is_empty() {
        return (vec![], vec![]);
    }

    let mut result = vec![];
    let names = schema.fields.iter().map(|f| f.name.clone()).collect();

    for rb in rbs {
        let mut cols = vec![];

        for array in rb.columns() {
            let array_ptr = Box::new(ffi::ArrowArray::empty());
            let schema_ptr = Box::new(ffi::ArrowSchema::empty());
            let array_ptr = Box::into_raw(array_ptr);
            let schema_ptr = Box::into_raw(schema_ptr);
            unsafe {
                ffi::export_field_to_c(
                    &Field::new("", array.data_type().clone(), true),
                    schema_ptr,
                );
                ffi::export_array_to_c(array.clone(), array_ptr);
            };
            cols.push((array_ptr as uintptr_t, schema_ptr as uintptr_t));
        }

        result.push(cols);
    }
    (names, result)
}
