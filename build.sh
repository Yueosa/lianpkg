#!/bin/bash
# ===========================================================
#   LianPkg 全量构建脚本
#   编译 CLI + .so + .dll + Linux GUI 并打包到 build/{version}/
#
#   Windows GUI 通过 GitHub Actions 构建，不在此脚本中
# ===========================================================

set -e

# =========================
# 🎨 颜色
# =========================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

info()    { echo -e "${CYAN}ℹ️  $*${NC}"; }
success() { echo -e "${GREEN}✅ $*${NC}"; }
warn()    { echo -e "${YELLOW}⚠️  $*${NC}"; }
error()   { echo -e "${RED}❌ $*${NC}"; exit 1; }

# =========================
# 1️⃣ 项目信息
# =========================
[ -f "Cargo.toml" ] || error "Cargo.toml not found — run from project root"

NAME=$(grep -m 1 '^name =' Cargo.toml | sed -E 's/name = "(.*)"/\1/')
VERSION=$(grep -m 1 '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')

echo
info "Project: ${BLUE}${NAME}${NC} v${VERSION}"
info "Targets: Linux CLI + .so | Windows .exe + .dll | Linux GUI"

DEST="build/${VERSION}"
mkdir -p "$DEST"

# =========================
# 2️⃣ Linux: CLI + .so
# =========================
echo
info "── Linux (x86_64) ──"
cargo build --release

cp "target/release/${NAME}" \
   "${DEST}/${NAME}_${VERSION}_linux_x86_64"
cp "target/release/lib${NAME}.so" \
   "${DEST}/lib${NAME}_${VERSION}_linux_x86_64.so"

success "CLI → ${NAME}_${VERSION}_linux_x86_64"
success ".so → lib${NAME}_${VERSION}_linux_x86_64.so"

# =========================
# 3️⃣ Windows: .exe + .dll (交叉编译)
# =========================
echo
info "── Windows (x86_64, cross) ──"

if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    warn "跳过: 未安装 x86_64-pc-windows-gnu target"
    warn "安装: rustup target add x86_64-pc-windows-gnu"
else
    # 图标嵌入
    ICON_FILE=$(ls *.ico 2>/dev/null | head -n 1)
    WIN_RUSTFLAGS=""
    if [ -n "$ICON_FILE" ]; then
        info "嵌入图标: ${ICON_FILE}"
        echo "id ICON \"${ICON_FILE}\"" > resources.rc
        if x86_64-w64-mingw32-windres resources.rc -o resources.o && \
           x86_64-w64-mingw32-ar rcs libresources.a resources.o 2>/dev/null; then
            WIN_RUSTFLAGS="-C link-arg=$(pwd)/resources.o"
        else
            warn "图标编译失败，跳过嵌入"
        fi
    fi

    RUSTFLAGS="$WIN_RUSTFLAGS" cargo build --release --target x86_64-pc-windows-gnu

    cp "target/x86_64-pc-windows-gnu/release/${NAME}.exe" \
       "${DEST}/${NAME}_${VERSION}_windows_x86_64.exe"
    cp "target/x86_64-pc-windows-gnu/release/${NAME}.dll" \
       "${DEST}/${NAME}_${VERSION}_windows_x86_64.dll"

    success ".exe → ${NAME}_${VERSION}_windows_x86_64.exe"
    success ".dll → ${NAME}_${VERSION}_windows_x86_64.dll"

    # 清理资源临时文件
    rm -f resources.rc resources.o libresources.a
fi

# =========================
# 4️⃣ Linux GUI (Flutter)
# =========================
echo
info "── Linux GUI ──"

if ! command -v flutter &>/dev/null; then
    warn "跳过: 未安装 Flutter SDK"
else
    BUNDLE="gui/build/linux/x64/release/bundle"

    pushd gui > /dev/null
    flutter build linux --release
    popd > /dev/null

    # 把 liblianpkg.so 复制到 bundle/lib/
    cp "target/release/lib${NAME}.so" "${BUNDLE}/lib/lib${NAME}.so"
    success "已将 lib${NAME}.so 打包进 GUI bundle"

    # 打包 tar.gz
    TAR_NAME="${NAME}-gui_${VERSION}_linux_x86_64.tar.gz"
    tar -czf "${DEST}/${TAR_NAME}" -C "${BUNDLE}" .

    success "GUI → ${TAR_NAME}"
fi

# =========================
# 5️⃣ 结果
# =========================
echo
success "构建完成！产物列表:"
ls -lh "${DEST}"
