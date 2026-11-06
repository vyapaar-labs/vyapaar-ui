//! Application router: maps the active route and handles navigation.

use serde::{Deserialize, Serialize};

/// The top-level routes of the workstation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppRoute {
    /// Home / watchlist dashboard.
    Home,
    /// Neural AST screener.
    Screener,
    /// Strategy Studio visual node canvas.
    Studio,
    /// Backtest simulator workbench.
    Simulator,
    /// System health panel.
    System,
}

impl AppRoute {
    /// The label used in navigation chrome.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Screener => "Screener",
            Self::Studio => "Studio",
            Self::Simulator => "Simulator",
            Self::System => "System",
        }
    }

    /// The path segment for this route (used by host routers).
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Screener => "/screener",
            Self::Studio => "/studio",
            Self::Simulator => "/simulator",
            Self::System => "/system",
        }
    }

    /// All routes in display order.
    pub const ALL: [AppRoute; 5] = [
        AppRoute::Home,
        AppRoute::Screener,
        AppRoute::Studio,
        AppRoute::Simulator,
        AppRoute::System,
    ];
}

/// Errors produced by the router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouterError {
    /// Cannot navigate from a route that is already active.
    AlreadyActive,
}

/// The router state: which route is active and where we came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Router {
    /// The active route.
    pub active: AppRoute,
    /// The route that was active before the last navigation.
    pub previous: Option<AppRoute>,
}

impl Router {
    /// Create a router starting at a route.
    pub fn new(initial: AppRoute) -> Self {
        Self {
            active: initial,
            previous: None,
        }
    }

    /// The default router, starting at [`AppRoute::Home`].
    pub fn default_router() -> Self {
        Self::new(AppRoute::Home)
    }

    /// Navigate to a new route, recording the previous one.
    ///
    /// # Errors
    /// [`RouterError::AlreadyActive`] when `route` equals the active route.
    pub fn navigate(&mut self, route: AppRoute) -> Result<(), RouterError> {
        if route == self.active {
            return Err(RouterError::AlreadyActive);
        }
        self.previous = Some(self.active);
        self.active = route;
        Ok(())
    }

    /// Navigate back to the previous route (stays put if there is none).
    pub fn back(&mut self) {
        if let Some(prev) = self.previous {
            self.active = prev;
            self.previous = None;
        }
    }
}
