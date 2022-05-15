mod gtk;

#[cfg(feature = "libadwaita")]
#[cfg_attr(dox, doc(cfg(feature = "libadwaita")))]
mod libadwaita;

#[cfg(feature = "libpanel")]
#[cfg_attr(dox, doc(cfg(feature = "libpanel")))]
mod libpanel;

#[cfg(test)]
mod tests;
