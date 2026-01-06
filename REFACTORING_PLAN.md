# Refactoring Plan: Serde-Based OpenAPI Generation

## Overview

Phase 1 (helper function extraction) is **COMPLETE** ✅. This document outlines Phase 2: replacing manual JSON string construction with proper serde-based serialization using the types defined in `src/openapi.rs`.

## Phase 1 Completion Summary

All 16 helper functions have been successfully extracted, tested, and integrated:
- ✅ Info & Tags helpers (4 functions)
- ✅ Path building helpers (4 functions)
- ✅ Schema & Components helpers (8 functions)
- ✅ Main `openapi_json` method refactored to use helpers
- ✅ 111 tests passing (16 new helper-specific tests added)
- ✅ Performance test confirms no regressions
- ✅ Comprehensive module documentation added

**Result:** The codebase is now well-structured and ready for serde migration.

## Phase 2: Serde Migration Strategy

### Goals

1. Replace all manual JSON string construction with serde serialization
2. Use the type-safe structs defined in `src/openapi.rs`
3. Maintain 100% compatibility with existing OpenAPI output
4. Improve type safety and maintainability
5. Enable easier future extensions (YAML output, OpenAPI 3.1, etc.)

### Current Architecture

**Manual JSON Construction (Phase 1):**
- Each helper function builds JSON strings with `format!()` macros
- String concatenation and escaping handled manually
- Error-prone and hard to maintain
- Example: `build_contact_json()` constructs `{"name":"...","url":"..."}` strings

**Target Architecture (Phase 2):**
- Each helper function returns a strongly-typed struct
- Serde handles all JSON serialization automatically
- Type safety enforced at compile time
- Example: `build_contact_json()` returns `Option<Contact>`

### Migration Steps

#### Step 1: Enhance `src/openapi.rs` Types

The existing `openapi.rs` has basic types but needs enhancements:

**Required Changes:**

1. **Add Serde derives to all types:**
   ```rust
   use serde::{Deserialize, Serialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct OpenAPI {
       pub openapi: String,
       pub info: Info,
       pub paths: HashMap<String, PathItem>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub components: Option<Components>,
       #[serde(skip_serializing_if = "Vec::is_empty", default)]
       pub tags: Vec<Tag>,
   }
   ```

2. **Expand existing types to match OpenAPI 3.0 spec:**
   - Add missing fields to `Info` (termsOfService, contact, license)
   - Add `Tag` type with external docs support
   - Expand `PathItem` to include all HTTP methods
   - Add proper `Response`, `RequestBody`, `Parameter` types
   - Add `SecurityScheme` for bearer token support

3. **Add serde attributes for OpenAPI conventions:**
   - `#[serde(rename = "camelCase")]` for field name transformations
   - `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
   - `#[serde(default)]` for fields with defaults
   - Custom serializers where needed

4. **Remove the `JsonValue` enum:**
   - Not needed with proper serde types
   - Replace with concrete types

**New types needed:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocs {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub scheme: String,
    #[serde(rename = "bearerFormat")]
    pub bearer_format: String,
}
```

#### Step 2: Update Helper Functions (Bottom-Up)

Migrate helper functions from returning `String` to returning typed structs.

**Migration Order (dependencies first):**

1. **Leaf functions (no dependencies):**
   - `build_contact_json() -> Option<Contact>`
   - `build_license_json() -> Option<License>`
   - `build_security_schemes_json() -> Option<HashMap<String, SecurityScheme>>`

2. **Mid-level functions:**
   - `build_info_json() -> Info`
   - `build_tags_json() -> Vec<Tag>`
   - `build_schemas_json() -> HashMap<String, Schema>`

3. **Composite functions:**
   - `build_components_json() -> Option<Components>`
   - `build_method_json() -> Operation`
   - `build_path_json() -> PathItem`

4. **Top-level functions:**
   - `build_paths_json() -> HashMap<String, PathItem>`
   - `openapi_json() -> String` (internally builds `OpenAPI` then serializes)

**Example Migration:**

```rust
// BEFORE (Phase 1):
fn build_contact_json(&self) -> String {
    let mut parts = Vec::new();
    if let Some(name) = &self.openapi.info.contact.as_ref().and_then(|c| c.0.as_ref()) {
        parts.push(format!(r#""name":"{}""#, name.replace('"', "\\\"")));
    }
    // ... more string building
    format!("{{{}}}", parts.join(","))
}

// AFTER (Phase 2):
fn build_contact(&self) -> Option<Contact> {
    self.openapi.info.contact.as_ref().map(|(name, url, email)| Contact {
        name: name.clone(),
        url: url.clone(),
        email: email.clone(),
    })
}
```

#### Step 3: Update Main `openapi_json()` Method

**Current approach:**
```rust
pub fn openapi_json(&self) -> String {
    let info_json = self.build_info_json();
    let paths_json = self.build_paths_json();
    let tags_json = self.build_tags_json();
    let components_json = self.build_components_json();
    
    format!(
        r#"{{"openapi":"3.0.0","info":{},"paths":{},"tags":{},"components":{}}}"#,
        info_json, paths_json, tags_json, components_json
    )
}
```

**Target approach:**
```rust
pub fn openapi_json(&self) -> String {
    let openapi = OpenAPI {
        openapi: "3.0.0".to_string(),
        info: self.build_info(),
        paths: self.build_paths(),
        tags: self.build_tags(),
        components: self.build_components(),
    };
    
    serde_json::to_string(&openapi).unwrap_or_else(|e| {
        eprintln!("Failed to serialize OpenAPI spec: {}", e);
        r#"{"openapi":"3.0.0","info":{"title":"Error","version":"0.0.0"},"paths":{}}"#.to_string()
    })
}
```

#### Step 4: Add Integration Tests

Create tests that verify the serde-based output matches the original string-based output:

```rust
#[cfg(test)]
mod serde_migration_tests {
    #[test]
    fn test_serde_output_matches_manual_json() {
        // Create identical router configurations
        let router = api_router!("Test API", "1.0.0")
            .description("Test description")
            .route("/test", get(test_handler));
        
        let json_output = router.openapi_json();
        
        // Parse and compare structure
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(parsed["openapi"], "3.0.0");
        assert_eq!(parsed["info"]["title"], "Test API");
        // ... more assertions
    }
    
    #[test]
    fn test_hello_world_schema_unchanged() {
        // Use the hello-world example
        // Compare against known good output from .scratch/openapi-refactored-branch.json
    }
}
```

#### Step 5: Update YAML Generation

Once serde is in place, YAML becomes trivial:

```rust
pub fn openapi_yaml(&self) -> String {
    let openapi = self.build_openapi_struct();
    serde_yaml::to_string(&openapi).unwrap_or_else(|e| {
        eprintln!("Failed to serialize OpenAPI spec to YAML: {}", e);
        "openapi: 3.0.0\ninfo:\n  title: Error\n  version: 0.0.0\npaths: {}".to_string()
    })
}
```

### Implementation Timeline

#### Week 1: Type System Foundation
- [ ] Add `serde` and `serde_json` dependencies to `Cargo.toml`
- [ ] Enhance `src/openapi.rs` with complete type definitions
- [ ] Add all necessary serde derives and attributes
- [ ] Create integration tests for type round-tripping

#### Week 2: Leaf Function Migration
- [ ] Migrate `build_contact_json` → `build_contact`
- [ ] Migrate `build_license_json` → `build_license`
- [ ] Migrate `build_security_schemes_json` → `build_security_schemes`
- [ ] Update tests for these functions
- [ ] Verify JSON output unchanged

#### Week 3: Mid-Level Function Migration
- [ ] Migrate `build_info_json` → `build_info`
- [ ] Migrate `build_tags_json` → `build_tags`
- [ ] Migrate `build_schemas_json` → `build_schemas`
- [ ] Update tests for these functions
- [ ] Run comparison tests against master branch

#### Week 4: Composite Function Migration
- [ ] Migrate `build_components_json` → `build_components`
- [ ] Migrate `build_method_json` → `build_method`
- [ ] Migrate `build_path_json` → `build_path`
- [ ] Update all related tests

#### Week 5: Top-Level Integration
- [ ] Migrate `build_paths_json` → `build_paths`
- [ ] Update `openapi_json` to use serde serialization
- [ ] Run full test suite (all 111+ tests must pass)
- [ ] Generate hello-world OpenAPI and compare with baseline

#### Week 6: Cleanup & Documentation
- [ ] Remove old string-building code
- [ ] Add YAML support using `serde_yaml`
- [ ] Update module documentation
- [ ] Add examples showing type-safe API
- [ ] Performance testing and optimization

### Testing Strategy

1. **Unit Tests:** Each migrated function has tests verifying structure
2. **Integration Tests:** Full OpenAPI generation compared to baseline
3. **Comparison Tests:** New output must match Phase 1 output exactly
4. **Golden File Tests:** Use `.scratch/openapi-refactored-branch.json` as reference
5. **Property Tests:** Verify round-trip serialization/deserialization

### Dependencies to Add

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
serde_yaml = "0.9"  # For YAML output support
```

### Rollback Plan

Since we're keeping the helper functions structure from Phase 1:

1. Each function migration is isolated and can be reverted independently
2. Tests will catch any breaking changes immediately
3. Git commits should be atomic (one function migration per commit)
4. If issues arise, revert to Phase 1 completed state (current `feat/mitch/refactor-monolith` branch)

### Success Criteria

- [ ] All 111+ existing tests pass
- [ ] Hello-world example generates identical JSON (modulo ordering)
- [ ] New type-safe API works correctly
- [ ] YAML output supported
- [ ] Code is more maintainable than Phase 1
- [ ] Performance is equal or better than Phase 1
- [ ] Documentation updated and complete

### Benefits of Serde Migration

1. **Type Safety:** Compile-time guarantees for OpenAPI structure
2. **Maintainability:** No more manual JSON escaping or formatting
3. **Extensibility:** Easy to add new fields or support OpenAPI 3.1
4. **Multi-Format:** YAML support comes for free
5. **Validation:** Serde ensures valid JSON structure
6. **Developer Experience:** Better IDE support and autocompletion

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking changes to output format | Comprehensive comparison tests against Phase 1 baseline |
| Performance regression | Benchmarking tests; serde is highly optimized |
| Type definition complexity | Incremental migration; extensive documentation |
| Serde attribute errors | Unit tests for each type; example-based testing |
| Field naming issues | Use `#[serde(rename)]` attributes; verify against spec |

### Future Enhancements (Post-Phase 2)

Once serde migration is complete, these become trivial to implement:

- OpenAPI 3.1 support (just update types)
- JSON Schema draft 2020-12
- Multiple server definitions
- Webhooks support
- Custom extensions (x-* fields)
- API versioning strategies
- Schema validation on generation

## Notes

- Phase 1 created a solid foundation for this migration
- The helper function structure makes the serde migration straightforward
- Each function can be migrated independently without breaking the build
- The `.scratch/openapi-refactored-branch.json` file serves as our golden reference
- All changes should be made on the `feat/mitch/refactor-monolith` branch
