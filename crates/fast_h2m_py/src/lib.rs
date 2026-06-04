use fast_h2m_core::{ConversionError, ConversionOptions};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyAny;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[pyclass]
struct MarkdownStreamProcessor {
    inner: fast_h2m_core::MarkdownStreamProcessor,
}

#[pymethods]
impl MarkdownStreamProcessor {
    #[new]
    #[pyo3(signature = (options = None))]
    fn new(py: Python<'_>, options: Option<Py<PyAny>>) -> PyResult<Self> {
        let options = parse_options(py, options)?;
        Ok(Self {
            inner: fast_h2m_core::MarkdownStreamProcessor::new(options),
        })
    }

    fn process_chunk(&mut self, chunk: &str) -> String {
        self.inner.process_chunk(chunk)
    }

    fn finish(&mut self) -> String {
        self.inner.finish()
    }
}

#[pyfunction]
#[pyo3(signature = (html, options = None))]
fn convert(py: Python<'_>, html: &str, options: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
    let options = parse_options(py, options)?;
    let result = fast_h2m_core::convert(html, options).map_err(conversion_error_to_py)?;
    let result_json = serde_json::to_string(&result).map_err(runtime_error)?;

    json_loads(py, &result_json)
}

#[pyfunction]
#[pyo3(signature = (html, options = None))]
fn convert_to_markdown(py: Python<'_>, html: &str, options: Option<Py<PyAny>>) -> PyResult<String> {
    let options = parse_options(py, options)?;
    let result = fast_h2m_core::convert(html, options).map_err(conversion_error_to_py)?;

    Ok(result.content.unwrap_or_default())
}

#[pymodule]
fn fast_h2m(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(convert_to_markdown, m)?)?;
    m.add_class::<MarkdownStreamProcessor>()?;
    Ok(())
}

fn parse_options(py: Python<'_>, options: Option<Py<PyAny>>) -> PyResult<ConversionOptions> {
    let Some(options) = options else {
        return Ok(ConversionOptions::default());
    };
    let options = options.bind(py);
    if options.is_none() {
        return Ok(ConversionOptions::default());
    }

    let options_json = json_dumps(py, options)?;
    serde_json::from_str(&options_json)
        .map_err(|err| PyValueError::new_err(format!("invalid conversion options: {err}")))
}

fn json_dumps(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<String> {
    py.import("json")?
        .call_method1("dumps", (value,))?
        .extract::<String>()
}

fn json_loads(py: Python<'_>, value: &str) -> PyResult<Py<PyAny>> {
    Ok(py.import("json")?.call_method1("loads", (value,))?.unbind())
}

fn conversion_error_to_py(error: ConversionError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn runtime_error(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}
