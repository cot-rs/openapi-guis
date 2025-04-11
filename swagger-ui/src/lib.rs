//! This crate implements necessary boilerplate code to serve [Swagger UI] via
//! web server. It provides a simple API to configure the [Swagger UI] and serve
//! it via a web server. The crate is deliberately kept simple and does not
//! implement any web server specific code. It is up to the user to
//! implement the web server specific code for the web framework of choice.
//!
//! It does not download Swagger UI from the internet, but rather includes the
//! necessary static files in the crate. This reduces the number of build
//! dependencies and makes it easy to use the crate offline.
//!
//! It was mainly created to be integrated inside the [Cot web framework](https://cot.rs/),
//! but does not depend on it. It can be used with any web framework.
//!
//! # Attribution
//!
//! This crate is heavily based on [`utoipa-swagger-ui`](https://github.com/juhaku/utoipa),
//! licensed under Apache 2.0/MIT.
//!
//! [Swagger UI] included in this crate is licensed under Apache 2.0.
//!
//! [Swagger UI]: https://swagger.io/tools/swagger-ui/

#![warn(missing_docs, rustdoc::missing_crate_level_docs)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::mem;

pub mod oauth;

use serde::Serialize;

/// Entry point for serving Swagger UI and api docs in application. It provides
/// builder style chainable configuration methods for configuring api doc urls.
///
/// # Examples
///
/// ```
/// # use swagger_ui_redist::SwaggerUi;
/// let mut swagger = SwaggerUi::new();
/// swagger.config().urls(["/api-docs/openapi.json"]);
/// let static_files = SwaggerUi::static_files(); // static files that are needed to be served
/// let html = swagger.serve()?;
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SwaggerUi {
    title: Cow<'static, str>,
    config: Config<'static>,
    file_paths: HashMap<SwaggerUiStaticFile, String>,
}

impl Default for SwaggerUi {
    fn default() -> Self {
        Self::new()
    }
}

impl SwaggerUi {
    /// Create a new [`SwaggerUi`] for given path.
    ///
    /// Path argument will expose the Swagger UI to the user and should be
    /// something that the underlying application framework / library
    /// supports.
    ///
    /// # Examples
    ///
    /// Exposes Swagger UI using path `/swagger-ui` using actix-web supported
    /// syntax.
    ///
    /// ```
    /// # use swagger_ui_redist::SwaggerUi;
    /// let swagger = SwaggerUi::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: Cow::Borrowed("Swagger UI"),
            config: Config::new(),
            file_paths: SwaggerUiStaticFile::default_map(),
        }
    }

    /// Return a mutable reference to the config, allowing to modify it.
    ///
    /// This is useful for setting up the Swagger UI with custom settings,
    /// but also for setting up the URLs for the OpenAPI spec.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::{SwaggerUi, Config};
    /// let mut swagger = SwaggerUi::new();
    /// swagger
    ///     .config()
    ///     .urls(["/api-docs/openapi.json"])
    ///     .try_it_out_enabled(true)
    ///     .filter(true);
    /// ```
    pub fn config(&mut self) -> &mut Config<'static> {
        &mut self.config
    }

    /// Sets the title for the Swagger UI page.
    ///
    /// The title will be displayed in the browser tab.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::SwaggerUi;
    /// let mut swagger = SwaggerUi::new();
    /// swagger.title("My API Documentation");
    /// ```
    pub fn title(&mut self, title: impl Into<Cow<'static, str>>) -> &mut Self {
        self.title = title.into();
        self
    }

    /// Returns a reference to all static files required by Swagger UI.
    ///
    /// This method provides access to the raw content of all static files
    /// needed to properly render the Swagger UI interface. Each file is
    /// paired with its corresponding [`SwaggerUiStaticFile`] enum variant
    /// for identification.
    ///
    /// # Returns
    ///
    /// A static slice of tuples containing the file identifier and its raw
    /// content.
    #[must_use]
    pub fn static_files() -> &'static [(SwaggerUiStaticFile, &'static [u8])] {
        &[
            (
                SwaggerUiStaticFile::Css,
                include_bytes!("../res/swagger-ui.css"),
            ),
            (
                SwaggerUiStaticFile::IndexCss,
                include_bytes!("../res/index.css"),
            ),
            (
                SwaggerUiStaticFile::Js,
                include_bytes!("../res/swagger-ui-bundle.js"),
            ),
            (
                SwaggerUiStaticFile::StandalonePresetJs,
                include_bytes!("../res/swagger-ui-standalone-preset.js"),
            ),
            (
                SwaggerUiStaticFile::Favicon16,
                include_bytes!("../res/favicon-16x16.png"),
            ),
            (
                SwaggerUiStaticFile::Favicon32,
                include_bytes!("../res/favicon-32x32.png"),
            ),
        ]
    }

    /// Overrides the path for a specific static file.
    ///
    /// This method allows customizing the URL paths where static files are
    /// served from. This is useful when integrating with web frameworks
    /// that have specific routing requirements or when serving files from a
    /// CDN or different location.
    ///
    /// # Parameters
    ///
    /// * `static_file` - The static file type to override
    /// * `path` - The new path where the file will be served from
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::{SwaggerUi, SwaggerUiStaticFile};
    /// let mut swagger = SwaggerUi::new();
    /// swagger.override_file_path(
    ///     SwaggerUiStaticFile::Css,
    ///     "/assets/swagger-ui.css".to_string(),
    /// );
    /// ```
    pub fn override_file_path(&mut self, static_file: SwaggerUiStaticFile, path: String) {
        self.file_paths.insert(static_file, path);
    }

    /// Generates the HTML for the Swagger UI page.
    ///
    /// This method creates a complete HTML document that includes all necessary
    /// CSS and JavaScript references to render the Swagger UI interface. The
    /// HTML is configured according to the settings specified in the
    /// [`Config`] object.
    ///
    /// # Returns
    ///
    /// A `Result` containing the HTML string if successful, or an error if the
    /// configuration could not be formatted properly.
    ///
    /// # Errors
    ///
    /// Returns an error if the Swagger UI config fails to be serialized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::SwaggerUi;
    /// # fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let mut swagger = SwaggerUi::new();
    /// swagger.config().urls(["/api-docs/openapi.json"]);
    /// let html = swagger.serve()?;
    /// # Ok(())
    /// # }
    /// ```
    #[expect(clippy::missing_panics_doc)]
    pub fn serve(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let title = &self.title;
        let css_path = self
            .file_paths
            .get(&SwaggerUiStaticFile::Css)
            .expect("all files should be present");
        let index_css_path = self
            .file_paths
            .get(&SwaggerUiStaticFile::IndexCss)
            .expect("all files should be present");
        let js_path = self
            .file_paths
            .get(&SwaggerUiStaticFile::Js)
            .expect("all files should be present");
        let standalone_preset_js_path = self
            .file_paths
            .get(&SwaggerUiStaticFile::StandalonePresetJs)
            .expect("all files should be present");

        let config = format_config(&self.config, DEFAULT_CONFIG)?;

        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="{css_path}" />
    <link rel="stylesheet" type="text/css" href="{index_css_path}" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="{js_path}" charset="UTF-8"></script>
<script src="{standalone_preset_js_path}" charset="UTF-8"></script>
<script>
    window.onload = () => {{
        {config}
    }};
</script>
</body>
</html>
"#
        ))
    }
}

/// Represents the static files required by Swagger UI.
///
/// This enum is used to identify and manage the various static assets needed
/// to properly render the Swagger UI interface.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SwaggerUiStaticFile {
    /// The main Swagger UI CSS file.
    Css,
    /// Additional index CSS styles for Swagger UI.
    IndexCss,
    /// The main Swagger UI JavaScript bundle.
    Js,
    /// The standalone preset JavaScript file for Swagger UI.
    StandalonePresetJs,
    /// The 16x16 favicon.
    Favicon16,
    /// The 32x32 favicon.
    Favicon32,
}

impl SwaggerUiStaticFile {
    /// Returns a slice containing all available Swagger UI static files.
    ///
    /// This method provides access to all the static files required by Swagger
    /// UI, including CSS, JavaScript, and favicon files.
    ///
    /// # Returns
    ///
    /// A static slice containing all variants of `SwaggerUiStaticFile`.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            SwaggerUiStaticFile::Css,
            SwaggerUiStaticFile::IndexCss,
            SwaggerUiStaticFile::Js,
            SwaggerUiStaticFile::StandalonePresetJs,
            SwaggerUiStaticFile::Favicon16,
            SwaggerUiStaticFile::Favicon32,
        ]
    }

    #[must_use]
    fn default_map() -> HashMap<Self, String> {
        let mut map = HashMap::new();

        for file in Self::all() {
            map.insert(*file, file.default_path());
        }

        map
    }

    #[must_use]
    fn default_path(self) -> String {
        format!("./{}", self.file_name())
    }

    /// Returns the filename for a specific static file.
    #[must_use]
    pub fn file_name(&self) -> &'static str {
        match self {
            SwaggerUiStaticFile::Css => "swagger-ui.css",
            SwaggerUiStaticFile::IndexCss => "index.css",
            SwaggerUiStaticFile::Js => "swagger-ui-bundle.js",
            SwaggerUiStaticFile::StandalonePresetJs => "swagger-ui-standalone-preset.js",
            SwaggerUiStaticFile::Favicon16 => "favicon-16x16.png",
            SwaggerUiStaticFile::Favicon32 => "favicon-32x32.png",
        }
    }
}

/// Rust type for Swagger UI url configuration object.
#[non_exhaustive]
#[derive(Debug, Default, Serialize, Clone)]
pub struct Url<'a> {
    name: Cow<'a, str>,
    #[allow(clippy::struct_field_names)]
    url: Cow<'a, str>,
    #[serde(skip)]
    primary: bool,
}

impl<'a> Url<'a> {
    /// Create new [`Url`].
    ///
    /// Name is shown in the select dropdown when there are multiple docs in
    /// Swagger UI.
    ///
    /// Url is path which exposes the OpenAPI doc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Url;
    /// let url = Url::new("My Api", "/api-docs/openapi.json");
    /// ```
    #[must_use]
    pub fn new(name: &'a str, url: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            url: Cow::Borrowed(url),
            ..Default::default()
        }
    }

    /// Create new [`Url`] with primary flag.
    ///
    /// Primary flag allows users to override the default behavior of the
    /// Swagger UI for selecting the primary doc to display. By default when
    /// there are multiple docs in Swagger UI the first one in the list will
    /// be the primary.
    ///
    /// Name is shown in the select dropdown when there are multiple docs in
    /// Swagger UI.
    ///
    /// Url is path which exposes the OpenAPI doc.
    ///
    /// # Examples
    ///
    /// Set "My Api" as primary.
    /// ```
    /// # use swagger_ui_redist::Url;
    /// let url = Url::with_primary("My Api", "/api-docs/openapi.json", true);
    /// ```
    #[must_use]
    pub fn with_primary(name: &'a str, url: &'a str, primary: bool) -> Self {
        Self {
            name: Cow::Borrowed(name),
            url: Cow::Borrowed(url),
            primary,
        }
    }
}

impl<'a> From<&'a str> for Url<'a> {
    fn from(url: &'a str) -> Self {
        Self {
            url: Cow::Borrowed(url),
            ..Default::default()
        }
    }
}

impl From<String> for Url<'_> {
    fn from(url: String) -> Self {
        Self {
            url: Cow::Owned(url),
            ..Default::default()
        }
    }
}

impl From<Cow<'static, str>> for Url<'_> {
    fn from(url: Cow<'static, str>) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

const SWAGGER_STANDALONE_LAYOUT: &str = "StandaloneLayout";
const SWAGGER_BASE_LAYOUT: &str = "BaseLayout";

/// Object used to alter Swagger UI settings.
///
/// Config struct provides [Swagger UI configuration](https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/configuration.md)
/// for settings which could be altered with **docker variables**.
///
/// # Examples
///
/// In simple case, create config directly from url that points to the api doc
/// json.
///
/// ```
/// # use swagger_ui_redist::Config;
/// let mut config = Config::new();
/// config.urls(["/api-docs/openapi.json"]);
/// ```
#[non_exhaustive]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config<'a> {
    /// Url to fetch external configuration from.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::struct_field_names)]
    config_url: Option<String>,

    /// Id of the DOM element where `Swagger UI` will put it's user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dom_id")]
    dom_id: Option<String>,

    /// [`Url`] the Swagger UI is serving.
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    /// Name of the primary url if any.
    #[serde(skip_serializing_if = "Option::is_none", rename = "urls.primaryName")]
    urls_primary_name: Option<String>,

    /// [`Url`]s the Swagger UI is serving.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    urls: Vec<Url<'a>>,

    /// Enables overriding configuration parameters with url query parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    query_config_enabled: Option<bool>,

    /// Controls whether [deep linking](https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/deep-linking.md)
    /// is enabled in OpenAPI spec.
    ///
    /// Deep linking automatically scrolls and expands UI to given url fragment.
    #[serde(skip_serializing_if = "Option::is_none")]
    deep_linking: Option<bool>,

    /// Controls whether operation id is shown in the operation list.
    #[serde(skip_serializing_if = "Option::is_none")]
    display_operation_id: Option<bool>,

    /// Default models expansion depth; -1 will completely hide the models.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_models_expand_depth: Option<isize>,

    /// Default model expansion depth from model example section.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_model_expand_depth: Option<isize>,

    /// Defines how models is show when API is first rendered.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_model_rendering: Option<String>,

    /// Define whether request duration in milliseconds is displayed for "Try it
    /// out" requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    display_request_duration: Option<bool>,

    /// Controls default expansion for operations and tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_expansion: Option<String>,

    /// Defines is filtering of tagged operations allowed with edit box in top
    /// bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<bool>,

    /// Controls how many tagged operations are shown. By default all operations
    /// are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_displayed_tags: Option<usize>,

    /// Defines whether extensions are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    show_extensions: Option<bool>,

    /// Defines whether common extensions are shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    show_common_extensions: Option<bool>,

    /// Defines whether "Try it out" section should be enabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    try_it_out_enabled: Option<bool>,

    /// Defines whether request snippets section is enabled. If disabled legacy
    /// curl snipped will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    request_snippets_enabled: Option<bool>,

    /// Oauth redirect url.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth2_redirect_url: Option<String>,

    /// Defines whether request mutated with `requestInterceptor` will be used
    /// to produce curl command in the UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    show_mutated_request: Option<bool>,

    /// Define supported http request submit methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_submit_methods: Option<Vec<String>>,

    /// Define validator url which is used to validate the Swagger spec. By
    /// default the validator swagger.io's online validator is used. Setting
    /// this to none will disable spec validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    validator_url: Option<String>,

    /// Enables passing credentials to CORS requests as defined
    /// [fetch standards](https://fetch.spec.whatwg.org/#credentials).
    #[serde(skip_serializing_if = "Option::is_none")]
    with_credentials: Option<bool>,

    /// Defines whether authorizations is persisted throughout browser refresh
    /// and close.
    #[serde(skip_serializing_if = "Option::is_none")]
    persist_authorization: Option<bool>,

    /// [`oauth::Config`] the Swagger UI is using for auth flow.
    #[serde(skip)]
    oauth: Option<oauth::Config>,

    /// Defines syntax highlighting specific options.
    #[serde(skip_serializing_if = "Option::is_none")]
    syntax_highlight: Option<SyntaxHighlight>,

    /// The layout of Swagger UI uses, default is `"StandaloneLayout"`.
    layout: &'a str,

    /// Basic authentication configuration. If configured, the Swagger UI will
    /// prompt for basic auth credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    basic_auth: Option<BasicAuth>,
}

impl<'a> Config<'a> {
    /// Constructs a new [`Config`] with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let config = Config::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the URLs for the OpenAPI specifications to be displayed in Swagger
    /// UI.
    ///
    /// This method accepts an iterator of items that can be converted into
    /// [`Url`] objects. It handles both single and multiple URL scenarios
    /// appropriately:
    /// - For a single URL, it will be set as the primary URL
    /// - For multiple URLs, they will be displayed in a dropdown selector
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    ///
    /// // Single URL
    /// config.urls(["/api-docs/openapi.json"]);
    ///
    /// // Multiple URLs
    /// config.urls(["/api-docs/openapi.json", "/api-docs/openapi-v2.json"]);
    /// ```
    pub fn urls<I: IntoIterator<Item = U>, U: Into<Url<'a>>>(&mut self, urls: I) -> &mut Self {
        let urls = urls.into_iter().map(Into::into).collect::<Vec<Url<'a>>>();
        let urls_len = urls.len();

        if urls_len == 1 {
            self.single_url(urls);
        } else {
            self.multiple_urls(urls);
        }

        self
    }

    fn multiple_urls(&mut self, urls: Vec<Url<'a>>) {
        let primary_name = urls
            .iter()
            .find(|url| url.primary)
            .map(|url| url.name.to_string());

        self.urls_primary_name = primary_name;
        self.urls = urls
            .into_iter()
            .map(|mut url| {
                if url.name.is_empty() {
                    url.name = Cow::Owned(String::from(&url.url[..]));

                    url
                } else {
                    url
                }
            })
            .collect();
    }

    fn single_url(&mut self, mut urls: Vec<Url<'a>>) {
        let url = urls.get_mut(0).map(mem::take).unwrap();
        let primary_name = if url.primary {
            Some(url.name.to_string())
        } else {
            None
        };

        self.urls_primary_name = primary_name;
        self.url = if url.name.is_empty() {
            Some(url.url.to_string())
        } else {
            None
        };
        self.urls = if url.name.is_empty() {
            Vec::new()
        } else {
            vec![url]
        };
    }

    /// Constructs a new [`Config`] from [`Iterator`] of [`Url`]s.
    ///
    /// # Examples
    /// Create new config with oauth config.
    /// ```
    /// # use swagger_ui_redist::{Config, oauth};
    /// let mut config = Config::new();
    /// let config = config.oauth_config(oauth::Config::new());
    /// ```
    pub fn oauth_config(&mut self, oauth_config: oauth::Config) -> &mut Self {
        self.oauth = Some(oauth_config);
        self
    }

    /// Add url to fetch external configuration from.
    ///
    /// # Examples
    ///
    /// Set external config url.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.config_url("http://url.to.external.config");
    /// ```
    pub fn config_url<S: Into<String>>(&mut self, config_url: S) -> &mut Self {
        self.config_url = Some(config_url.into());

        self
    }

    /// Add id of the DOM element where `Swagger UI` will put it's user
    /// interface.
    ///
    /// The default value is `#swagger-ui`.
    ///
    /// # Examples
    ///
    /// Set custom dom id where the Swagger UI will place it's content.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.dom_id("#my-id");
    /// ```
    pub fn dom_id<S: Into<String>>(&mut self, dom_id: S) -> &mut Self {
        self.dom_id = Some(dom_id.into());

        self
    }

    /// Set `query_config_enabled` to allow overriding configuration parameters
    /// via url `query` parameters.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    ///
    /// Enable query config.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.query_config_enabled(true);
    /// ```
    pub fn query_config_enabled(&mut self, query_config_enabled: bool) -> &mut Self {
        self.query_config_enabled = Some(query_config_enabled);

        self
    }

    /// Set `deep_linking` to allow deep linking tags and operations.
    ///
    /// Deep linking will automatically scroll to and expand operation when
    /// Swagger UI is given corresponding url fragment. See more at
    /// [deep linking docs](https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/deep-linking.md).
    ///
    /// Deep linking is enabled by default.
    ///
    /// # Examples
    ///
    /// Disable the deep linking.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.deep_linking(false);
    /// ```
    pub fn deep_linking(&mut self, deep_linking: bool) -> &mut Self {
        self.deep_linking = Some(deep_linking);

        self
    }

    /// Set `display_operation_id` to `true` to show operation id in the
    /// operations list.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    ///
    /// Allow operation id to be shown.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.display_operation_id(true);
    /// ```
    pub fn display_operation_id(&mut self, display_operation_id: bool) -> &mut Self {
        self.display_operation_id = Some(display_operation_id);

        self
    }

    /// Set 'layout' to '`BaseLayout`' to only use the base swagger layout
    /// without a search header.
    ///
    /// Default value is '`StandaloneLayout`'.
    ///
    /// # Examples
    ///
    /// Configure Swagger to use Base Layout instead of Standalone
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.use_base_layout();
    /// ```
    pub fn use_base_layout(&mut self) -> &mut Self {
        self.layout = SWAGGER_BASE_LAYOUT;

        self
    }

    /// Add default models expansion depth.
    ///
    /// Setting this to `-1` will completely hide the models.
    ///
    /// # Examples
    ///
    /// Hide all the models.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.default_models_expand_depth(-1);
    /// ```
    pub fn default_models_expand_depth(&mut self, default_models_expand_depth: isize) -> &mut Self {
        self.default_models_expand_depth = Some(default_models_expand_depth);

        self
    }

    /// Add default model expansion depth for model on the example section.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.default_model_expand_depth(1);
    /// ```
    pub fn default_model_expand_depth(&mut self, default_model_expand_depth: isize) -> &mut Self {
        self.default_model_expand_depth = Some(default_model_expand_depth);

        self
    }

    /// Add `default_model_rendering` to set how models is show when API is
    /// first rendered.
    ///
    /// The user can always switch the rendering for given model by clicking the
    /// `Model` and `Example Value` links.
    ///
    /// * `example` Makes example rendered first by default.
    /// * `model` Makes model rendered first by default.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.default_model_rendering(r#"["example"*, "model"]"#);
    /// ```
    pub fn default_model_rendering<S: Into<String>>(
        &mut self,
        default_model_rendering: S,
    ) -> &mut Self {
        self.default_model_rendering = Some(default_model_rendering.into());

        self
    }

    /// Set to `true` to show request duration of _**'Try it out'**_ requests
    /// _**(in milliseconds)**_.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    /// Enable request duration of the _**'Try it out'**_ requests.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.display_request_duration(true);
    /// ```
    pub fn display_request_duration(&mut self, display_request_duration: bool) -> &mut Self {
        self.display_request_duration = Some(display_request_duration);

        self
    }

    /// Add `doc_expansion` to control default expansion for operations and
    /// tags.
    ///
    /// * `list` Will expand only tags.
    /// * `full` Will expand tags and operations.
    /// * `none` Will expand nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.doc_expansion(r#"["list"*, "full", "none"]"#);
    /// ```
    pub fn doc_expansion<S: Into<String>>(&mut self, doc_expansion: S) -> &mut Self {
        self.doc_expansion = Some(doc_expansion.into());

        self
    }

    /// Add `filter` to allow filtering of tagged operations.
    ///
    /// When enabled top bar will show and edit box that can be used to filter
    /// visible tagged operations. Filter behaves case sensitive manner and
    /// matches anywhere inside the tag.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    ///
    /// Enable filtering.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.filter(true);
    /// ```
    pub fn filter(&mut self, filter: bool) -> &mut Self {
        self.filter = Some(filter);

        self
    }

    /// Add `max_displayed_tags` to restrict shown tagged operations.
    ///
    /// By default all operations are shown.
    ///
    /// # Examples
    ///
    /// Display only 4 operations.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.max_displayed_tags(4);
    /// ```
    pub fn max_displayed_tags(&mut self, max_displayed_tags: usize) -> &mut Self {
        self.max_displayed_tags = Some(max_displayed_tags);

        self
    }

    /// Set `show_extensions` to adjust whether vendor extension _**`(x-)`**_
    /// fields and values are shown for operations, parameters, responses
    /// and schemas.
    ///
    /// # Example
    ///
    /// Show vendor extensions.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.show_extensions(true);
    /// ```
    pub fn show_extensions(&mut self, show_extensions: bool) -> &mut Self {
        self.show_extensions = Some(show_extensions);

        self
    }

    /// Add `show_common_extensions` to define whether common extension
    /// _**`(pattern, maxLength, minLength, maximum, minimum)`**_ fields and
    /// values are shown for parameters.
    ///
    /// # Examples
    ///
    /// Show common extensions.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.show_common_extensions(true);
    /// ```
    pub fn show_common_extensions(&mut self, show_common_extensions: bool) -> &mut Self {
        self.show_common_extensions = Some(show_common_extensions);

        self
    }

    /// Add `try_it_out_enabled` to enable _**'Try it out'**_ section by
    /// default.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    ///
    /// Enable _**'Try it out'**_ section by default.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.try_it_out_enabled(true);
    /// ```
    pub fn try_it_out_enabled(&mut self, try_it_out_enabled: bool) -> &mut Self {
        self.try_it_out_enabled = Some(try_it_out_enabled);

        self
    }

    /// Set `request_snippets_enabled` to enable request snippets section.
    ///
    /// If disabled legacy curl snipped will be used.
    ///
    /// Default value is `false`.
    ///
    /// # Examples
    ///
    /// Enable request snippets section.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.request_snippets_enabled(true);
    /// ```
    pub fn request_snippets_enabled(&mut self, request_snippets_enabled: bool) -> &mut Self {
        self.request_snippets_enabled = Some(request_snippets_enabled);

        self
    }

    /// Add oauth redirect url.
    ///
    /// # Examples
    ///
    /// Add oauth redirect url.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.oauth2_redirect_url("http://my.oauth2.redirect.url");
    /// ```
    pub fn oauth2_redirect_url<S: Into<String>>(&mut self, oauth2_redirect_url: S) -> &mut Self {
        self.oauth2_redirect_url = Some(oauth2_redirect_url.into());

        self
    }

    /// Add `show_mutated_request` to use request returned from
    /// `requestInterceptor` to produce curl command in the UI. If set to
    /// `false` the request before `requestInterceptor` was applied will be
    /// used.
    ///
    /// # Examples
    ///
    /// Use request after `requestInterceptor` to produce the curl command.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.show_mutated_request(true);
    /// ```
    pub fn show_mutated_request(&mut self, show_mutated_request: bool) -> &mut Self {
        self.show_mutated_request = Some(show_mutated_request);

        self
    }

    /// Add supported http methods for _**'Try it out'**_ operation.
    ///
    /// _**'Try it out'**_ will be enabled based on the given list of http
    /// methods when the operation's http method is included within the
    /// list. By giving an empty list will disable _**'Try it out'**_ from
    /// all operations but it will **not** filter operations from the UI.
    ///
    /// By default all http operations are enabled.
    ///
    /// # Examples
    ///
    /// Set allowed http methods explicitly.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.supported_submit_methods([
    ///     "get", "put", "post", "delete", "options", "head", "patch", "trace",
    /// ]);
    /// ```
    ///
    /// Allow _**'Try it out'**_ for only GET operations.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.supported_submit_methods(["get"]);
    /// ```
    pub fn supported_submit_methods<I: IntoIterator<Item = S>, S: Into<String>>(
        &mut self,
        supported_submit_methods: I,
    ) -> &mut Self {
        self.supported_submit_methods = Some(
            supported_submit_methods
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        self
    }

    /// Add validator url which is used to validate the Swagger spec.
    ///
    /// This can also be set to use locally deployed validator for example see
    /// [Validator Badge](https://github.com/swagger-api/validator-badge) for more details.
    ///
    /// By default swagger.io's online validator _**`(https://validator.swagger.io/validator)`**_ will be used.
    /// Setting this to `none` will disable the validator.
    ///
    /// # Examples
    ///
    /// Disable the validator.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.validator_url("none");
    /// ```
    pub fn validator_url<S: Into<String>>(&mut self, validator_url: S) -> &mut Self {
        self.validator_url = Some(validator_url.into());

        self
    }

    /// Set `with_credentials` to enable passing credentials to CORS requests
    /// send by browser as defined [fetch standards](https://fetch.spec.whatwg.org/#credentials).
    ///
    /// **Note!** that Swagger UI cannot currently set cookies cross-domain
    /// (see [swagger-js#1163](https://github.com/swagger-api/swagger-js/issues/1163)) -
    /// as a result, you will have to rely on browser-supplied cookies (which
    /// this setting enables sending) that Swagger UI cannot control.
    ///
    /// # Examples
    ///
    /// Enable passing credentials to CORS requests.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.with_credentials(true);
    /// ```
    pub fn with_credentials(&mut self, with_credentials: bool) -> &mut Self {
        self.with_credentials = Some(with_credentials);

        self
    }

    /// Set to `true` to enable authorizations to be persisted throughout
    /// browser refresh and close.
    ///
    /// Default value is `false`.
    ///
    ///
    /// # Examples
    ///
    /// Persists authorization throughout browser close and refresh.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.persist_authorization(true);
    /// ```
    pub fn persist_authorization(&mut self, persist_authorization: bool) -> &mut Self {
        self.persist_authorization = Some(persist_authorization);

        self
    }

    /// Set a specific configuration for syntax highlighting responses
    /// and curl commands.
    ///
    /// By default, swagger-ui does syntax highlighting of responses
    /// and curl commands.  This may consume considerable resources in
    /// the browser when executed on large responses.
    ///
    /// # Example
    ///
    /// Disable syntax highlighting.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// let mut config = Config::new();
    /// config.with_syntax_highlight(false);
    /// ```
    pub fn with_syntax_highlight<H: Into<SyntaxHighlight>>(
        &mut self,
        syntax_highlight: H,
    ) -> &mut Self {
        self.syntax_highlight = Some(syntax_highlight.into());

        self
    }

    /// Set basic authentication configuration.
    /// If configured, the Swagger UI will prompt for basic auth credentials.
    /// username and password are required. "{username}:{password}" will be
    /// base64 encoded and added to the "Authorization" header.
    /// If not provided or wrong credentials are provided, the user will be
    /// prompted again. # Examples
    ///
    /// Configure basic authentication.
    /// ```
    /// # use swagger_ui_redist::Config;
    /// # use swagger_ui_redist::BasicAuth;
    /// let mut config = Config::new();
    /// config.basic_auth(BasicAuth {
    ///     username: "admin".to_string(),
    ///     password: "password".to_string(),
    /// });
    /// ```
    pub fn basic_auth(&mut self, basic_auth: BasicAuth) -> &mut Self {
        self.basic_auth = Some(basic_auth);

        self
    }
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            config_url: Option::default(),
            dom_id: Some("#swagger-ui".to_string()),
            url: Option::default(),
            urls_primary_name: Option::default(),
            urls: Vec::default(),
            query_config_enabled: Option::default(),
            deep_linking: Some(true),
            display_operation_id: Option::default(),
            default_models_expand_depth: Option::default(),
            default_model_expand_depth: Option::default(),
            default_model_rendering: Option::default(),
            display_request_duration: Option::default(),
            doc_expansion: Option::default(),
            filter: Option::default(),
            max_displayed_tags: Option::default(),
            show_extensions: Option::default(),
            show_common_extensions: Option::default(),
            try_it_out_enabled: Option::default(),
            request_snippets_enabled: Option::default(),
            oauth2_redirect_url: Option::default(),
            show_mutated_request: Option::default(),
            supported_submit_methods: Option::default(),
            validator_url: Option::default(),
            with_credentials: Option::default(),
            persist_authorization: Option::default(),
            oauth: Option::default(),
            syntax_highlight: Option::default(),
            layout: SWAGGER_STANDALONE_LAYOUT,
            basic_auth: Option::default(),
        }
    }
}

/// Basic auth options for Swagger UI. By providing `BasicAuth` to
/// `Config::basic_auth` the access to the Swagger UI can be restricted behind
/// given basic authentication.
#[derive(Debug, Serialize, Clone)]
pub struct BasicAuth {
    /// Username for the `BasicAuth`
    pub username: String,
    /// Password of the _`username`_ for the `BasicAuth`
    pub password: String,
}

/// Represents settings related to syntax highlighting of payloads and
/// cURL commands.
#[derive(Debug, Serialize, Clone)]
#[non_exhaustive]
pub struct SyntaxHighlight {
    /// Boolean telling whether syntax highlighting should be
    /// activated or not. Defaults to `true`.
    pub activated: bool,
    /// Highlight.js syntax coloring theme to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<&'static str>,
}

impl Default for SyntaxHighlight {
    fn default() -> Self {
        Self {
            activated: true,
            theme: None,
        }
    }
}

impl From<bool> for SyntaxHighlight {
    fn from(value: bool) -> Self {
        Self {
            activated: value,
            ..Default::default()
        }
    }
}

impl SyntaxHighlight {
    /// Explicitly specifies whether syntax highlighting is to be
    /// activated or not.  Defaults to true.
    #[must_use]
    pub fn activated(mut self, activated: bool) -> Self {
        self.activated = activated;
        self
    }

    /// Explicitly specifies the
    /// [Highlight.js](https://highlightjs.org/) coloring theme to
    /// utilize for syntax highlighting.
    #[must_use]
    pub fn theme(mut self, theme: &'static str) -> Self {
        self.theme = Some(theme);
        self
    }
}

/// Represents servable file of Swagger UI. This is used together with [`serve`]
/// function to serve Swagger UI files via web server.
#[non_exhaustive]
#[derive(Debug)]
pub struct SwaggerFile<'a> {
    /// Content of the file as [`Cow`] [`slice`] of bytes.
    pub bytes: Cow<'a, [u8]>,
    /// Content type of the file e.g `"text/xml"`.
    pub content_type: String,
}

#[inline]
fn format_config(config: &Config<'_>, file: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let config_json = match serde_json::to_string_pretty(&config) {
        Ok(config) => config,
        Err(error) => return Err(Box::new(error)),
    };

    // Replace {{config}} with pretty config json and remove the curly brackets `{
    // }` from beginning and the end.
    Ok(file.replace("{{config}}", &config_json[2..&config_json.len() - 2]))
}

const DEFAULT_CONFIG: &str = r"
window.ui = SwaggerUIBundle({
  {{config}},
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});";

#[cfg(test)]
mod tests {
    use similar::TextDiff;

    use super::*;

    fn assert_diff_equal(expected: &str, new: &str) {
        let diff = TextDiff::from_lines(expected, new);

        assert_eq!(expected, new, "\nDifference:\n{}", diff.unified_diff());
    }

    const TEST_INITIAL_CONFIG: &str = r"
window.ui = SwaggerUIBundle({
  {{config}},
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});";

    #[test]
    fn format_swagger_config_json_single_url() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "url": "/api-docs/openapi1.json",
  "deepLinking": true,
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new().urls(["/api-docs/openapi1.json"]),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_json_single_url_with_name() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "urls": [
    {
      "name": "api-doc1",
      "url": "/api-docs/openapi1.json"
    }
  ],
  "deepLinking": true,
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new().urls([Url::new("api-doc1", "/api-docs/openapi1.json")]),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_json_single_url_primary() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "urls.primaryName": "api-doc1",
  "urls": [
    {
      "name": "api-doc1",
      "url": "/api-docs/openapi1.json"
    }
  ],
  "deepLinking": true,
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new().urls([Url::with_primary(
                "api-doc1",
                "/api-docs/openapi1.json",
                true,
            )]),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_multiple_urls_with_primary() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "urls.primaryName": "api-doc1",
  "urls": [
    {
      "name": "api-doc1",
      "url": "/api-docs/openapi1.json"
    },
    {
      "name": "api-doc2",
      "url": "/api-docs/openapi2.json"
    }
  ],
  "deepLinking": true,
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new().urls([
                Url::with_primary("api-doc1", "/api-docs/openapi1.json", true),
                Url::new("api-doc2", "/api-docs/openapi2.json"),
            ]),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_multiple_urls() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "urls": [
    {
      "name": "/api-docs/openapi1.json",
      "url": "/api-docs/openapi1.json"
    },
    {
      "name": "/api-docs/openapi2.json",
      "url": "/api-docs/openapi2.json"
    }
  ],
  "deepLinking": true,
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new().urls(["/api-docs/openapi1.json", "/api-docs/openapi2.json"]),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_with_multiple_fields() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#another-el",
  "url": "/api-docs/openapi1.json",
  "queryConfigEnabled": true,
  "deepLinking": false,
  "displayOperationId": true,
  "defaultModelsExpandDepth": 1,
  "defaultModelExpandDepth": -1,
  "defaultModelRendering": "[\"example\"*]",
  "displayRequestDuration": true,
  "docExpansion": "[\"list\"*]",
  "filter": true,
  "maxDisplayedTags": 1,
  "showExtensions": true,
  "showCommonExtensions": true,
  "tryItOutEnabled": true,
  "requestSnippetsEnabled": true,
  "oauth2RedirectUrl": "http://auth",
  "showMutatedRequest": true,
  "supportedSubmitMethods": [
    "get"
  ],
  "validatorUrl": "none",
  "withCredentials": true,
  "persistAuthorization": true,
  "layout": "BaseLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new()
                .urls(["/api-docs/openapi1.json"])
                .deep_linking(false)
                .dom_id("#another-el")
                .default_model_expand_depth(-1)
                .default_model_rendering(r#"["example"*]"#)
                .default_models_expand_depth(1)
                .display_operation_id(true)
                .display_request_duration(true)
                .filter(true)
                .use_base_layout()
                .doc_expansion(r#"["list"*]"#)
                .max_displayed_tags(1)
                .oauth2_redirect_url("http://auth")
                .persist_authorization(true)
                .query_config_enabled(true)
                .request_snippets_enabled(true)
                .show_common_extensions(true)
                .show_extensions(true)
                .show_mutated_request(true)
                .supported_submit_methods(["get"])
                .try_it_out_enabled(true)
                .validator_url("none")
                .with_credentials(true),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_with_syntax_highlight_default() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "url": "/api-docs/openapi1.json",
  "deepLinking": true,
  "syntaxHighlight": {
    "activated": true
  },
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new()
                .urls(["/api-docs/openapi1.json"])
                .with_syntax_highlight(SyntaxHighlight::default()),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_with_syntax_highlight_on() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "url": "/api-docs/openapi1.json",
  "deepLinking": true,
  "syntaxHighlight": {
    "activated": true
  },
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new()
                .urls(["/api-docs/openapi1.json"])
                .with_syntax_highlight(true),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_with_syntax_highlight_off() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "url": "/api-docs/openapi1.json",
  "deepLinking": true,
  "syntaxHighlight": {
    "activated": false
  },
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new()
                .urls(["/api-docs/openapi1.json"])
                .with_syntax_highlight(false),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }

    #[test]
    fn format_swagger_config_with_syntax_highlight_default_with_theme() {
        const EXPECTED: &str = r##"
window.ui = SwaggerUIBundle({
    "dom_id": "#swagger-ui",
  "url": "/api-docs/openapi1.json",
  "deepLinking": true,
  "syntaxHighlight": {
    "activated": true,
    "theme": "monokai"
  },
  "layout": "StandaloneLayout",
  presets: [
    SwaggerUIBundle.presets.apis,
    SwaggerUIStandalonePreset
  ],
  plugins: [
    SwaggerUIBundle.plugins.DownloadUrl
  ],
});"##;

        let formatted_config = match format_config(
            Config::new()
                .urls(["/api-docs/openapi1.json"])
                .with_syntax_highlight(SyntaxHighlight::default().theme("monokai")),
            TEST_INITIAL_CONFIG,
        ) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };

        assert_diff_equal(EXPECTED, &formatted_config);
    }
}
