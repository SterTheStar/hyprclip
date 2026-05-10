#!/usr/bin/env bash
set -euo pipefail

APP="hyprclip"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
ARCH=$(uname -m)
BUILD_DIR="target/release"
DIST_DIR="dist"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

echo "==> Building $APP $VERSION (release)..."
cargo build --release

# ──────────────────────────────────────
# tar.xz
# ──────────────────────────────────────
echo "==> Packaging $APP-$VERSION-$ARCH.tar.xz..."
TAR_DIR="$DIST_DIR/$APP-$VERSION"
mkdir -p "$TAR_DIR"
cp "$BUILD_DIR/$APP" "$TAR_DIR/"
tar -cJf "$DIST_DIR/$APP-$VERSION-$ARCH.tar.xz" -C "$DIST_DIR" "$APP-$VERSION"
rm -rf "$TAR_DIR"
echo "    $DIST_DIR/$APP-$VERSION-$ARCH.tar.xz"

# ──────────────────────────────────────
# deb
# ──────────────────────────────────────
echo "==> Building .deb package..."
if command -v cargo-deb &>/dev/null; then
    cargo-deb --no-build --output "$DIST_DIR/"
    echo "    $(ls "$DIST_DIR"/*.deb)"
else
    DEB_DIR="$DIST_DIR/${APP}_${VERSION}_${ARCH}"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/bin"

    cp "$BUILD_DIR/$APP" "$DEB_DIR/usr/bin/"
    strip "$DEB_DIR/usr/bin/$APP"

    cat > "$DEB_DIR/DEBIAN/control" <<EOF
Package: $APP
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Depends: libgtk-4-1, libadwaita-1-0, libgtk4-layer-shell-0
Maintainer: SterTheStar <ster@hyprclip>
Description: A clipboard history manager for Hyprland
 A clipboard history manager for Hyprland with Wayland layer-shell support.
EOF

    dpkg-deb --root-owner-group --build "$DEB_DIR" "$DIST_DIR/${APP}_${VERSION}_${ARCH}.deb"
    rm -rf "$DEB_DIR"
    echo "    $DIST_DIR/${APP}_${VERSION}_${ARCH}.deb"
fi

# ──────────────────────────────────────
# rpm
# ──────────────────────────────────────
echo "==> Building .rpm package..."
if command -v cargo-generate-rpm &>/dev/null; then
    cargo-generate-rpm --output "$DIST_DIR/"
    echo "    $(ls "$DIST_DIR"/*.rpm)"
else
    RPM_TOPDIR="$DIST_DIR/rpmbuild"
    mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    cat > "$RPM_TOPDIR/SPECS/$APP.spec" <<EOF
Name:           $APP
Version:        $VERSION
Release:        1%{?dist}
Summary:        A clipboard history manager for Hyprland
License:        MIT
URL:            https://github.com/SterTheStar/hyprclip

Requires:       gtk4 >= 4.0
Requires:       libadwaita >= 1.0
Requires:       gtk4-layer-shell >= 0.5

%description
A clipboard history manager for Hyprland with Wayland layer-shell support.

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 $BUILD_DIR/$APP %{buildroot}/usr/bin/$APP

%files
/usr/bin/$APP

%changelog
EOF

    rpmbuild --define "_topdir $RPM_TOPDIR" -bb "$RPM_TOPDIR/SPECS/$APP.spec" 2>/dev/null || {
        echo "    [!] rpmbuild not available, skipping .rpm"
    }
    if ls "$RPM_TOPDIR"/RPMS/*/*.rpm &>/dev/null; then
        cp "$RPM_TOPDIR"/RPMS/*/*.rpm "$DIST_DIR/"
        echo "    $(ls "$DIST_DIR"/*.rpm)"
    fi
    rm -rf "$RPM_TOPDIR"
fi

echo ""
echo "==> Done! Packages in $DIST_DIR/"
ls -lh "$DIST_DIR/"
