#!/usr/bin/env bash
# shellcheck shell=bash

set -euo pipefail

release_mode="${RELEASE_MODE:?RELEASE_MODE is required}"
release_tag="${RELEASE_TAG:?RELEASE_TAG is required}"
dist_dir="${DIST_DIR:-dist}"

package_binary() {
  local target="${TARGET:?TARGET is required}"
  local bin_name="${BIN_NAME:?BIN_NAME is required}"
  local archive_kind="${ARCHIVE_KIND:?ARCHIVE_KIND is required}"
  local version="${release_tag#v}"
  local asset="scaffold-${version}-${target}"

  mkdir -p "${dist_dir}/${asset}"
  cp "target/${target}/release/${bin_name}" "${dist_dir}/${asset}/"
  cp README.md LICENSE "${dist_dir}/${asset}/"

  if [[ "$archive_kind" == "zip" ]]; then
    7z a "${dist_dir}/${asset}.zip" "./${dist_dir}/${asset}/*"
  else
    tar -C "$dist_dir" -czf "${dist_dir}/${asset}.tar.gz" "$asset"
  fi
}

release_files() {
  shopt -s nullglob
  files=("${dist_dir}"/*)

  if (( ${#files[@]} == 0 )); then
    echo "No release assets found in ${dist_dir}." >&2
    exit 1
  fi
}

publish_tag_release() {
  release_files

  if gh release view "$release_tag" >/dev/null 2>&1; then
    gh release upload "$release_tag" "${files[@]}" --clobber
  else
    gh release create "$release_tag" "${files[@]}" --verify-tag --generate-notes
  fi
}

publish_rolling_release() {
  release_files

  local release_title="${RELEASE_TITLE:?RELEASE_TITLE is required}"
  local notes_file
  notes_file="$(mktemp)"
  trap 'rm -f "$notes_file"' RETURN

  {
    echo "Automated rolling build for ${GITHUB_SHA}."
    echo
    echo "Commit: ${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/commit/${GITHUB_SHA}"
  } > "$notes_file"

  git config user.name "github-actions[bot]"
  git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  git tag --force "$release_tag" "$GITHUB_SHA"
  git push --force origin "refs/tags/${release_tag}"

  local release_exists=false
  if gh release view "$release_tag" >/dev/null 2>&1; then
    release_exists=true
    gh release edit "$release_tag" \
      --title "$release_title" \
      --notes-file "$notes_file" \
      --prerelease \
      --latest=false
  else
    gh release create "$release_tag" \
      --title "$release_title" \
      --notes-file "$notes_file" \
      --prerelease \
      --latest=false \
      --verify-tag
  fi

  if [[ "$release_exists" == "true" ]]; then
    mapfile -t existing_assets < <(gh release view "$release_tag" --json assets --jq '.assets[].name')
    for asset_name in "${existing_assets[@]}"; do
      gh release delete-asset "$release_tag" "$asset_name" --yes
    done
  fi

  gh release upload "$release_tag" "${files[@]}"
}

case "$release_mode" in
  package-binary)
    package_binary
    ;;
  tag)
    publish_tag_release
    ;;
  rolling)
    publish_rolling_release
    ;;
  *)
    echo "Unknown RELEASE_MODE: ${release_mode}" >&2
    exit 1
    ;;
esac
