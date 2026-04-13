# Maintainer: Dxsk <https://github.com/Dxsk>
pkgname=git-identity-manager
pkgver=1.0.0 # x-release-please-version
pkgrel=1
pkgdesc="Simple CLI tool to switch between Git identities per repository using fzf"
arch=('any')
url="https://github.com/Dxsk/git-identity-manager"
license=('MIT')
depends=('bash' 'jq' 'fzf' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/Dxsk/git-identity-manager/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 git-identity.sh "$pkgdir/usr/bin/git-identity"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
