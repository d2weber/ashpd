//! # Examples
//!
//! ## Taking a screenshot
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let uri = screenshot::take(Default::default(), true, true).await?;
//!     println!("URI: {}", uri);
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!
//!     let uri = proxy.screenshot(Default::default(), true, true).await?;
//!     println!("URI: {}", uri);
//!     Ok(())
//! }
//! ```
//!
//! ## Picking a color
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let color = screenshot::pick_color(Default::default()).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```
//!
//! Or by using the Proxy directly
//!
//! ```rust,no_run
//! use ashpd::desktop::screenshot::ScreenshotProxy;
//!
//! async fn run() -> Result<(), ashpd::Error> {
//!     let connection = zbus::azync::Connection::new_session().await?;
//!     let proxy = ScreenshotProxy::new(&connection).await?;
//!
//!     let color = proxy.pick_color(Default::default()).await?;
//!     println!("({}, {}, {})", color.red(), color.green(), color.blue());
//!
//!     Ok(())
//! }
//! ```
//!

use super::{HandleToken, DESTINATION, PATH};
use crate::{helpers::call_request_method, Error, WindowIdentifier};
use zvariant_derive::{DeserializeDict, SerializeDict, TypeDict};

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::screenshot`] request.
struct ScreenshotOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
    /// Whether the dialog should be modal.
    modal: Option<bool>,
    /// Hint whether the dialog should offer customization before taking a
    /// screenshot.
    interactive: Option<bool>,
}

impl ScreenshotOptions {
    /// Sets whether the dialog should be a modal.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = Some(modal);
        self
    }

    /// Sets whether the dialog should offer customization before a screenshot
    /// or not.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = Some(interactive);
        self
    }
}

#[derive(DeserializeDict, SerializeDict, Clone, TypeDict, Debug)]
/// A response to a [`ScreenshotProxy::screenshot`] request.
struct Screenshot {
    /// The screenshot uri.
    uri: String,
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Clone, Debug, Default)]
/// Specified options for a [`ScreenshotProxy::pick_color`] request.
struct PickColorOptions {
    /// A string that will be used as the last element of the handle.
    handle_token: HandleToken,
}

#[derive(SerializeDict, DeserializeDict, Clone, Copy, PartialEq, TypeDict)]
/// A response to a [`ScreenshotProxy::pick_color`] request.
/// **Note** the values are normalized.
pub struct Color {
    color: ([f64; 3]),
}

impl Color {
    /// Red.
    pub fn red(&self) -> f64 {
        self.color[0]
    }

    /// Green.
    pub fn green(&self) -> f64 {
        self.color[1]
    }

    /// Blue.
    pub fn blue(&self) -> f64 {
        self.color[2]
    }
}

#[cfg(feature = "feature_gtk3")]
impl From<Color> for gtk3::gdk::RGBA {
    fn from(color: Color) -> gtk3::gdk::RGBA {
        gtk3::gdk::RGBA {
            red: color.red(),
            green: color.green(),
            blue: color.blue(),
            alpha: 1.0,
        }
    }
}

#[cfg(feature = "feature_gtk4")]
impl From<Color> for gtk4::gdk::RGBA {
    fn from(color: Color) -> gtk4::gdk::RGBA {
        gtk4::gdk::RGBA {
            red: color.red() as f32,
            green: color.green() as f32,
            blue: color.blue() as f32,
            alpha: 1_f32,
        }
    }
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Color")
            .field("red", &self.red())
            .field("green", &self.green())
            .field("blue", &self.blue())
            .finish()
    }
}

/// The interface lets sandboxed applications request a screenshot.
///
/// Wrapper of the DBus interface: [`org.freedesktop.portal.Screenshot`](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html#gdbus-org.freedesktop.portal.Screenshot).
#[derive(Debug)]
#[doc(alias = "org.freedesktop.portal.Screenshot")]
pub struct ScreenshotProxy<'a>(zbus::azync::Proxy<'a>);

impl<'a> ScreenshotProxy<'a> {
    /// Create a new instance of [`ScreenshotProxy`].
    pub async fn new(connection: &zbus::azync::Connection) -> Result<ScreenshotProxy<'a>, Error> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.portal.Screenshot")
            .path(PATH)?
            .destination(DESTINATION)
            .build_async()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::azync::Proxy<'_> {
        &self.0
    }

    /// Obtains the color of a single pixel.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    #[doc(alias = "PickColor")]
    pub async fn pick_color(&self, identifier: WindowIdentifier) -> Result<Color, Error> {
        let options = PickColorOptions::default();
        call_request_method(
            &self.0,
            &options.handle_token,
            "PickColor",
            &(identifier, &options),
        )
        .await
    }

    /// Takes a screenshot.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Identifier for the application window.
    /// * `interactive` - Sets whether the dialog should offer customization before a screenshot or not.
    /// * `modal` - Sets whether the dialog should be a modal.
    ///
    /// # Returns
    ///
    /// The screenshot URI.
    #[doc(alias = "Screenshot")]
    pub async fn screenshot(
        &self,
        identifier: WindowIdentifier,
        interactive: bool,
        modal: bool,
    ) -> Result<String, Error> {
        let options = ScreenshotOptions::default()
            .interactive(interactive)
            .modal(modal);
        let response: Screenshot = call_request_method(
            &self.0,
            &options.handle_token,
            "Screenshot",
            &(identifier, &options),
        )
        .await?;
        Ok(response.uri)
    }
}

#[doc(alias = "xdp_portal_pick_color")]
/// A handy wrapper around [`ScreenshotProxy::pick_color`].
pub async fn pick_color(identifier: WindowIdentifier) -> Result<Color, Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    proxy.pick_color(identifier).await
}

#[doc(alias = "xdp_portal_take_screenshot")]
/// A handy wrapper around [`ScreenshotProxy::screenshot`].
pub async fn take(
    identifier: WindowIdentifier,
    interactive: bool,
    modal: bool,
) -> Result<String, Error> {
    let connection = zbus::azync::Connection::new_session().await?;
    let proxy = ScreenshotProxy::new(&connection).await?;
    proxy.screenshot(identifier, interactive, modal).await
}
