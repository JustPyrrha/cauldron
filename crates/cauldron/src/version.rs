/// A Decima game's version.
pub type GameVersion = (u32, u32, u32, u32);

/// A plugin's runtime game version compatibility.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RuntimeVersion {
    /// Can run on any game version.
    VersionIndependent,
    /// Can only run on the provided game version.
    Strictly(GameVersion),
    /// Can run on any game version from the provided and up.
    AtLeast(GameVersion),
}

/// A plugin's game compatibility.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameType {
    /// Any Decima game.
    ///
    /// WARNING: Using this will also effectively set your plugin to [RuntimeVersion::VersionIndependent].
    GameIndependent,
    /// Horizon: Forbidden West
    HorizonForbiddenWest,
}
