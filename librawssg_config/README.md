# librawssg_config

**Version**: 1.0.0 (implied)  
**Crate name**: `librawssg_config`  
**Description**: Defines configuration data structures for the `librawssg` static site generator. Provides `Config`, `SiteConfig`, `BuildConfig`, `ContentRule`, and `NavItem` types, along with serialization/deserialization support (YAML and JSON) and validation logic.

---

## Table of Contents

1. [Overview](#overview)
2. [Modules](#modules)
3. [`BuildConfig`](#struct-buildconfig)
   - [Fields](#buildconfig-fields)
   - [Constructor `new()`](#buildconfig-new)
   - [`Default` Implementation](#buildconfig-default)
   - [Serialization & Deserialization](#buildconfig-serialization)
4. [`ContentRule`](#struct-contentrule)
   - [Fields](#contentrule-fields)
   - [Constructor `new()`](#contentrule-new)
   - [`Default` Implementation](#contentrule-default)
   - [Serialization & Deserialization](#contentrule-serialization)
5. [`NavItem`](#struct-navitem)
   - [Fields](#navitem-fields)
   - [Constructor `new()`](#navitem-new)
   - [`Default` Implementation](#navitem-default)
   - [Serialization & Deserialization](#navitem-serialization)
6. [`SiteConfig`](#struct-siteconfig)
   - [Fields](#siteconfig-fields)
   - [Constructor `new()`](#siteconfig-new)
   - [`Default` Implementation](#siteconfig-default)
   - [Serialization & Deserialization](#siteconfig-serialization)
7. [`Config`](#struct-config)
   - [Fields](#config-fields)
   - [Constructor `new()`](#config-new)
   - [Builder Method `with_site_name()`](#config-with_site_name)
   - [Rule Management Methods](#config-rule-management)
     - [`add_content_rule()`](#config-add_content_rule)
     - [`find_rule_by_name()`](#config-find_rule_by_name)
     - [`remove_rule_by_name()`](#config-remove_rule_by_name)
     - [`has_duplicate_rule_names()`](#config-has_duplicate_rule_names)
   - [Validation Method `validate()`](#config-validate)
   - [Serialization Methods](#config-serialization)
     - [`from_yaml_str()`](#config-from_yaml_str)
     - [`to_yaml_string()`](#config-to_yaml_string)
     - [`from_json_str()`](#config-from_json_str)
     - [`to_json_string()`](#config-to_json_string)
8. [Error Handling](#error-handling)
9. [Serialization Details](#serialization-details)
10. [Examples from Tests](#examples-from-tests)
11. [Testing Suite Overview](#testing-suite-overview)
12. [Conclusion](#conclusion)

---

## Overview

`librawssg_config` provides the central configuration types used by the static site generator. The main `Config` struct combines site settings (`SiteConfig`), build paths (`BuildConfig`), content processing rules (`ContentRule`), and arbitrary extra data. All types are serializable/deserializable via `serde`, enabling configuration to be read from and written to YAML or JSON files.

The types are designed with sensible defaults and include a `validate()` method to ensure the configuration is internally consistent and safe (e.g., preventing path traversal in patterns).

---

## Modules

The crate root (`lib.rs`) declares the following public modules:

- `build` – Contains `BuildConfig`.
- `config` – Contains `Config`.
- `content_rule` – Contains `ContentRule`.
- `nav` – Contains `NavItem`.
- `site` – Contains `SiteConfig`.

All public types are re‑exported at the crate root for convenience:

```rust
pub use build::BuildConfig;
pub use config::Config;
pub use content_rule::ContentRule;
pub use nav::NavItem;
pub use site::SiteConfig;
```

---

## Struct `BuildConfig`

Represents filesystem path configuration for the build process.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BuildConfig {
    #[serde(default = "default_content_dir")]
    pub content_dir: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_templates_dir")]
    pub templates_dir: String,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
}
```

### BuildConfig Fields

| Field           | Type     | Default Value | Description                                            |
| --------------- | -------- | ------------- | ------------------------------------------------------ |
| `content_dir`   | `String` | `"content"`   | Directory containing source content files.             |
| `output_dir`    | `String` | `"dist"`      | Directory where generated site output will be written. |
| `templates_dir` | `String` | `"templates"` | Directory containing template files.                   |
| `static_dir`    | `String` | `"static"`    | Directory containing static assets (copied as-is).     |

**Note**: `#[non_exhaustive]` prevents external crates from exhaustively matching or constructing with a struct literal. Use the provided constructors or update syntax.

### BuildConfig::new

```rust
#[must_use]
pub fn new() -> Self
```

**Purpose**: Creates a `BuildConfig` with default values (identical to `BuildConfig::default()`).

**Returns**: A new `BuildConfig` with all fields set to their defaults.

**Example**:

```rust
let build = BuildConfig::new();
assert_eq!(build.content_dir, "content");
```

### BuildConfig Default

The `Default` trait is implemented with the following values:

- `content_dir`: `"content"`
- `output_dir`: `"dist"`
- `templates_dir`: `"templates"`
- `static_dir`: `"static"`

These defaults can be overridden during deserialization; missing fields in serialized data will fall back to these defaults (thanks to `#[serde(default = "...")]`).

### BuildConfig Serialization

`BuildConfig` derives `Serialize` and `Deserialize`. When deserializing from YAML/JSON, any omitted fields will use the specified default functions. This allows partial configuration.

**Example** (from tests):

```rust
let yaml = "content_dir: custom_content\noutput_dir: public\n";
let build: BuildConfig = serde_yaml::from_str(yaml)?;
assert_eq!(build.content_dir, "custom_content");
assert_eq!(build.templates_dir, "templates"); // default
```

---

## Struct `ContentRule`

Defines how a certain group of content files should be processed.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ContentRule {
    pub name: String,
    pub pattern: String,
    pub template: String,
    #[serde(default)]
    pub list_template: Option<String>,
    #[serde(default)]
    pub list_enabled: bool,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### ContentRule Fields

| Field           | Type                                 | Default   | Description                                                                     |
| --------------- | ------------------------------------ | --------- | ------------------------------------------------------------------------------- |
| `name`          | `String`                             | `""`      | Unique identifier for the rule (e.g., `"blog"`, `"page"`).                      |
| `pattern`       | `String`                             | `""`      | Glob pattern matching content files (e.g., `"**/*.md"`). Must not contain `..`. |
| `template`      | `String`                             | `""`      | Name of the template to use for rendering each matched file.                    |
| `list_template` | `Option<String>`                     | `None`    | Optional template name for rendering list pages (e.g., index pages).            |
| `list_enabled`  | `bool`                               | `false`   | Whether list generation is enabled for this rule.                               |
| `extra`         | `HashMap<String, serde_json::Value>` | empty map | Arbitrary extra data associated with the rule.                                  |

### ContentRule::new

```rust
#[must_use]
pub fn new(
    name: impl Into<String>,
    pattern: impl Into<String>,
    template: impl Into<String>,
) -> Self
```

**Purpose**: Creates a `ContentRule` with the required fields (`name`, `pattern`, `template`). All other fields are set to their defaults.

**Parameters**:

- `name`: The rule name (converted to `String`).
- `pattern`: The glob pattern (converted to `String`).
- `template`: The template name (converted to `String`).

**Returns**: A new `ContentRule` instance.

**Example**:

```rust
let rule = ContentRule::new("blog", "**/*.md", "post");
assert_eq!(rule.name, "blog");
assert!(!rule.list_enabled);
```

### ContentRule Default

The `Default` implementation (derived) sets:

- `name`, `pattern`, `template`: empty strings
- `list_template`: `None`
- `list_enabled`: `false`
- `extra`: empty map

### ContentRule Serialization

Serializes/deserializes with `serde`. Missing optional fields default as specified. The `extra` map can hold any JSON‑compatible values.

**Example**:

```rust
let mut rule = ContentRule::new("page", "**/*.html", "base");
rule.list_enabled = true;
rule.list_template = Some("list".into());
rule.extra.insert("key".into(), json!("value"));
let yaml = serde_yaml::to_string(&rule)?;
let parsed: ContentRule = serde_yaml::from_str(&yaml)?;
assert_eq!(rule, parsed);
```

---

## Struct `NavItem`

Represents an item in a navigation menu (navbar or sidebar).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct NavItem {
    pub label: String,
    pub url: String,
    pub children: Vec<Self>,
}
```

### NavItem Fields

| Field      | Type           | Default | Description                                    |
| ---------- | -------------- | ------- | ---------------------------------------------- |
| `label`    | `String`       | `""`    | Display text for the navigation link.          |
| `url`      | `String`       | `""`    | URL the link points to (relative or absolute). |
| `children` | `Vec<NavItem>` | empty   | Nested sub‑items, enabling hierarchical menus. |

### NavItem::new

```rust
#[must_use]
pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self
```

**Purpose**: Creates a `NavItem` with a label and URL. The `children` vector starts empty.

**Parameters**:

- `label`: Display label.
- `url`: Target URL.

**Returns**: A new `NavItem`.

**Example**:

```rust
let item = NavItem::new("Home", "/");
assert_eq!(item.label, "Home");
assert!(item.children.is_empty());
```

### NavItem Default

`NavItem::default()` creates an item with empty label, empty URL, and no children.

### NavItem Serialization

Supports serialization and deserialization via `serde`. Nested children are handled recursively.

**Example**:

```rust
let parent = NavItem::new("Docs", "/docs");
parent.children.push(NavItem::new("API", "/docs/api"));
let json = serde_json::to_string(&parent)?;
let parsed: NavItem = serde_json::from_str(&json)?;
assert_eq!(parent, parsed);
```

---

## Struct `SiteConfig`

Holds global site metadata and navigation structures.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SiteConfig {
    #[serde(default)]
    pub navbar: Vec<super::NavItem>,
    #[serde(default)]
    pub sidebar: Vec<super::NavItem>,
    #[serde(default = "default_site_name")]
    pub site_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_language")]
    pub language: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### SiteConfig Fields

| Field         | Type                                 | Default       | Description                                                            |
| ------------- | ------------------------------------ | ------------- | ---------------------------------------------------------------------- |
| `navbar`      | `Vec<NavItem>`                       | empty         | Navigation items for the top bar.                                      |
| `sidebar`     | `Vec<NavItem>`                       | empty         | Navigation items for the sidebar.                                      |
| `site_name`   | `String`                             | `"librawssg"` | The name of the website.                                               |
| `description` | `Option<String>`                     | `None`        | Short site description.                                                |
| `language`    | `Option<String>`                     | `Some("en")`  | Site language code (e.g., `"en"`, `"id"`).                             |
| `base_url`    | `Option<String>`                     | `None`        | Base URL for the site; must start with `http://` or `https://` if set. |
| `author`      | `Option<String>`                     | `None`        | Default author name.                                                   |
| `repo_url`    | `Option<String>`                     | `None`        | URL to the source repository.                                          |
| `license`     | `Option<String>`                     | `None`        | License identifier (e.g., `"MIT"`).                                    |
| `extra`       | `HashMap<String, serde_json::Value>` | empty map     | Arbitrary extra site‑wide metadata.                                    |

### SiteConfig::new

```rust
#[must_use]
pub fn new(site_name: impl Into<String>) -> Self
```

**Purpose**: Creates a `SiteConfig` with a custom site name. All other fields are set to their defaults (navbar/sidebar empty, language `Some("en")`, etc.).

**Parameters**:

- `site_name`: The site name (converted to `String`).

**Returns**: A new `SiteConfig`.

**Example**:

```rust
let site = SiteConfig::new("My Site");
assert_eq!(site.site_name, "My Site");
assert_eq!(site.language.as_deref(), Some("en"));
```

### SiteConfig Default

`SiteConfig::default()` sets:

- `site_name`: `"librawssg"`
- `language`: `Some("en")`
- All `Option` fields: `None`
- Vectors and map: empty

### SiteConfig Serialization

Supports YAML/JSON. Missing fields during deserialization use defaults. The `language` default is provided by a custom function.

**Example**:

```rust
let mut site = SiteConfig::new("Test");
site.extra.insert("foo".into(), json!("bar"));
let yaml = serde_yaml::to_string(&site)?;
let parsed: SiteConfig = serde_yaml::from_str(&yaml)?;
assert_eq!(site, parsed);
```

---

## Struct `Config`

The top‑level configuration combining all other components.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct Config {
    pub site: SiteConfig,
    pub build: BuildConfig,
    pub content_rules: Vec<ContentRule>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### Config Fields

| Field           | Type                                 | Default                                         | Description                       |
| --------------- | ------------------------------------ | ----------------------------------------------- | --------------------------------- |
| `site`          | `SiteConfig`                         | `SiteConfig::default()` (site name "librawssg") | Global site configuration.        |
| `build`         | `BuildConfig`                        | `BuildConfig::default()`                        | Build path settings.              |
| `content_rules` | `Vec<ContentRule>`                   | empty                                           | List of content processing rules. |
| `extra`         | `HashMap<String, serde_json::Value>` | empty map                                       | Arbitrary top‑level extra data.   |

### Config::new

```rust
#[must_use]
pub fn new() -> Self
```

**Purpose**: Creates a `Config` with all fields defaulted. Equivalent to `Config::default()`.

**Returns**: A new `Config`.

**Example**:

```rust
let config = Config::new();
assert!(config.content_rules.is_empty());
```

### Config::with_site_name

```rust
#[must_use]
pub fn with_site_name(mut self, name: impl Into<String>) -> Self
```

**Purpose**: Builder‑style method that sets the `site.site_name` and returns the modified `Config`.

**Parameters**:

- `name`: The new site name.

**Returns**: The same `Config` with updated site name.

**Example**:

```rust
let config = Config::new().with_site_name("My Awesome Site");
assert_eq!(config.site.site_name, "My Awesome Site");
```

### Config Rule Management

#### `add_content_rule`

```rust
pub fn add_content_rule(&mut self, rule: ContentRule)
```

**Purpose**: Appends a `ContentRule` to the `content_rules` vector.

**Parameters**:

- `rule`: The rule to add.

**Example**:

```rust
config.add_content_rule(ContentRule::new("blog", "**/*.md", "post"));
```

#### `find_rule_by_name`

```rust
#[must_use]
pub fn find_rule_by_name(&self, name: &str) -> Option<&ContentRule>
```

**Purpose**: Searches for a content rule by its `name` field.

**Parameters**:

- `name`: The rule name to search for.

**Returns**: `Some(&ContentRule)` if found, otherwise `None`.

**Example**:

```rust
if let Some(rule) = config.find_rule_by_name("blog") {
    // ...
}
```

#### `remove_rule_by_name`

```rust
pub fn remove_rule_by_name(&mut self, name: &str) -> Option<ContentRule>
```

**Purpose**: Removes and returns the first content rule whose `name` matches the given string.

**Parameters**:

- `name`: The name of the rule to remove.

**Returns**: `Some(ContentRule)` if found and removed, otherwise `None`.

**Example**:

```rust
let removed = config.remove_rule_by_name("blog");
```

#### `has_duplicate_rule_names`

```rust
#[must_use]
pub fn has_duplicate_rule_names(&self) -> bool
```

**Purpose**: Checks whether any two content rules share the same `name`.

**Returns**: `true` if duplicates exist, `false` otherwise.

**Implementation**: Uses a `HashSet` to detect duplicates; O(n) time.

**Example**:

```rust
if config.has_duplicate_rule_names() {
    // handle error
}
```

### Config::validate

```rust
pub fn validate(&self) -> Result<()>
```

**Purpose**: Performs comprehensive validation of the configuration. Returns `Ok(())` if the configuration is valid, otherwise an `Err(Error::Validation(...))` with a descriptive message.

**Validation Rules**:

1. `site.site_name` must not be empty or whitespace‑only.
2. At least one content rule must be defined.
3. No duplicate content rule names.
4. For each content rule (indexed from 0):
   - `name` must not be empty or whitespace‑only.
   - `pattern` must not be empty or whitespace‑only.
   - `pattern` must not contain the substring `".."` (to prevent path traversal).
   - `template` must not be empty or whitespace‑only.
5. If `site.base_url` is `Some`, it must start with `"http://"` or `"https://"`.

**Returns**:

- `Ok(())` if all checks pass.
- `Err(Error::Validation(message))` on the first failure encountered.

**Example** (from tests):

```rust
let config = valid_config(); // has one rule
assert!(config.validate().is_ok());
```

### Config Serialization

The `Config` struct can be serialized to and deserialized from YAML and JSON via convenience methods.

#### `from_yaml_str`

```rust
pub fn from_yaml_str(yaml: &str) -> Result<Self>
```

**Purpose**: Parses a YAML string into a `Config`.

**Parameters**:

- `yaml`: YAML content as a string.

**Returns**:

- `Ok(Config)` on success.
- `Err(Error::Config)` if the YAML is invalid (with the underlying `serde_yaml` error message included).

**Example**:

```rust
let config = Config::from_yaml_str("site:\n  site_name: Test\n")?;
```

#### `to_yaml_string`

```rust
pub fn to_yaml_string(&self) -> Result<String>
```

**Purpose**: Serializes the `Config` to a YAML string.

**Returns**:

- `Ok(String)` with YAML representation.
- `Err(Error::Serialization)` if serialization fails.

#### `from_json_str`

```rust
pub fn from_json_str(json: &str) -> Result<Self>
```

**Purpose**: Parses a JSON string into a `Config`.

**Parameters**:

- `json`: JSON content as a string.

**Returns**:

- `Ok(Config)` on success.
- `Err(Error::Config)` if the JSON is invalid.

#### `to_json_string`

```rust
pub fn to_json_string(&self) -> Result<String>
```

**Purpose**: Serializes the `Config` to a JSON string.

**Returns**:

- `Ok(String)` with JSON representation.
- `Err(Error::Serialization)` on failure.

**Example roundtrip**:

```rust
let yaml = config.to_yaml_string()?;
let parsed = Config::from_yaml_str(&yaml)?;
assert_eq!(config, parsed);
```

---

## Error Handling

The `Config` methods and `validate` use `librawssg_error::Result<T>` (alias for `std::result::Result<T, librawssg_error::Error>`). The relevant error variants used in this crate are:

- `Error::Config` – For YAML/JSON deserialization failures.
- `Error::Serialization` – For serialization failures.
- `Error::Validation` – For `validate()` failures.

All error messages are descriptive and include context (e.g., which rule is invalid, what condition was violated).

---

## Serialization Details

All configuration structs derive `Serialize` and `Deserialize` from `serde`. Default values are applied during deserialization for missing fields via `#[serde(default = "function")]` or `#[serde(default)]` (which uses `Default::default()` for the field type).

- `BuildConfig`: Each field has a custom default function.
- `ContentRule`: Optional fields use `#[serde(default)]`.
- `NavItem`: No special defaults; all fields are required in input, but `Default` is derived for programmatic creation.
- `SiteConfig`: `site_name` has a custom default, `language` has a custom default returning `Some("en")`, others use `#[serde(default)]`.
- `Config`: `site` and `build` are required in YAML/JSON (they don't have `#[serde(default)]` at the field level, but the struct itself derives `Default` and the fields are not marked optional; however, when deserializing a top‑level `Config`, missing `site` or `build` will cause an error because they are not optional. In practice, configuration files should include these sections or rely on the `Default` implementation when constructing programmatically). **Important**: The `Config` struct's fields are not marked with `#[serde(default)]`, so during deserialization, missing `site` or `build` will cause a parse error. Users must provide at least `site` and `build` keys (can be empty maps to get defaults via the inner structs' own defaults). The `content_rules` and `extra` fields have `#[serde(default)]` so they can be omitted.

---

## Examples from Tests

The test suite provides extensive examples for each type. Below are selected snippets.

### BuildConfig

```rust
let build = BuildConfig::default();
assert_eq!(build.output_dir, "dist");

let yaml = "content_dir: custom_content\noutput_dir: public\n";
let build: BuildConfig = serde_yaml::from_str(yaml)?;
assert_eq!(build.content_dir, "custom_content");
assert_eq!(build.templates_dir, "templates");
```

### ContentRule

```rust
let mut rule = ContentRule::new("page", "**/*.html", "base");
rule.list_enabled = true;
rule.list_template = Some("list".into());
rule.extra.insert("key".into(), json!("value"));
```

### NavItem

```rust
let mut parent = NavItem::new("Docs", "/docs");
parent.children.push(NavItem::new("API", "/docs/api"));
```

### SiteConfig

```rust
let site = SiteConfig::new("My Site");
assert_eq!(site.language.as_deref(), Some("en"));
```

### Config Validation

```rust
let mut config = Config::new().with_site_name("My Site");
config.add_content_rule(ContentRule::new("page", "**/*.html", "base"));
assert!(config.validate().is_ok());

config.site.base_url = Some("ftp://example.com".to_string());
assert!(config.validate().is_err());
```

---

## Testing Suite Overview

The crate includes five test files:

- `build_tests.rs` – Tests `BuildConfig` defaults, `new()`, and YAML roundtrip.
- `config_tests.rs` – Extensive tests for `Config`: creation, rule management, validation (all rules), YAML/JSON roundtrip, and error cases.
- `content_rule_tests.rs` – Tests `ContentRule` constructor, defaults, and serialization.
- `nav_tests.rs` – Tests `NavItem` constructor, defaults, children, and serialization.
- `site_tests.rs` – Tests `SiteConfig` constructor, defaults, and serialization.

All tests are self‑contained and use the `must!` macro to unwrap results with a helpful message on failure. They serve as executable examples of the API usage.

---

## Conclusion

`librawssg_config` provides a clean and extensible configuration system for a static site generator. With sensible defaults, comprehensive validation, and full serde support, it covers the needs of both simple and complex site configurations. The types are designed for ergonomic use and can be easily loaded from YAML or JSON files, making it straightforward to define site‑wide settings, build paths, navigation, and content processing rules.

For further details, refer to the source code and test files.
