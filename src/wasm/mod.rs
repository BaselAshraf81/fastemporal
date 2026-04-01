/// WASM/JavaScript bindings via `wasm-bindgen`.
///
/// Enabled with the `wasm` Cargo feature:
/// ```toml
/// fastemporal = { version = "0.1", features = ["wasm"] }
/// ```

#[cfg(feature = "wasm")]
mod wasm_impl {
    use wasm_bindgen::prelude::*;
    use crate::{ZonedDateTime as Zdt, Duration};

    /// A [`ZonedDateTime`] exposed to JavaScript.
    #[wasm_bindgen]
    pub struct WasmZonedDateTime {
        inner: Zdt,
    }

    #[wasm_bindgen]
    impl WasmZonedDateTime {
        /// Returns the current time in UTC.
        #[wasm_bindgen(js_name = now)]
        pub fn now() -> WasmZonedDateTime {
            WasmZonedDateTime { inner: Zdt::now() }
        }

        /// Parse from an ISO 8601 string.
        #[wasm_bindgen(js_name = fromISO)]
        pub fn from_iso(s: &str) -> Result<WasmZonedDateTime, JsValue> {
            Zdt::from_iso(s)
                .map(|inner| WasmZonedDateTime { inner })
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        #[wasm_bindgen(getter)] pub fn year(&self)   -> i32 { self.inner.year() }
        #[wasm_bindgen(getter)] pub fn month(&self)  -> u8  { self.inner.month() }
        #[wasm_bindgen(getter)] pub fn day(&self)    -> u8  { self.inner.day() }
        #[wasm_bindgen(getter)] pub fn hour(&self)   -> u8  { self.inner.hour() }
        #[wasm_bindgen(getter)] pub fn minute(&self) -> u8  { self.inner.minute() }
        #[wasm_bindgen(getter)] pub fn second(&self) -> u8  { self.inner.second() }

        /// Add `days` calendar days.
        #[wasm_bindgen(js_name = plusDays)]
        pub fn plus_days(&self, days: i32) -> WasmZonedDateTime {
            WasmZonedDateTime { inner: self.inner.plus(Duration::days(days)) }
        }

        /// Subtract `days` calendar days.
        #[wasm_bindgen(js_name = minusDays)]
        pub fn minus_days(&self, days: i32) -> WasmZonedDateTime {
            WasmZonedDateTime { inner: self.inner.minus(Duration::days(days)) }
        }

        /// Convert to another IANA timezone.
        #[wasm_bindgen(js_name = inTimezone)]
        pub fn in_timezone(&self, tz: &str) -> Result<WasmZonedDateTime, JsValue> {
            self.inner
                .in_timezone(tz)
                .map(|inner| WasmZonedDateTime { inner })
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        /// ISO 8601 string.
        #[wasm_bindgen(js_name = toISO)]
        pub fn to_iso(&self) -> String {
            self.inner.to_iso()
        }

        /// Format using a strftime/Luxon format string.
        #[wasm_bindgen]
        pub fn format(&self, fmt: &str) -> String {
            self.inner.format(fmt)
        }
    }
}
