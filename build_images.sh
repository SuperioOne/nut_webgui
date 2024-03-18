#!/usr/bin/bash
set -e;

IMAGE_NAME="${IMAGE_NAME:-"nut_webgui"}"
REGISTRY="${REGISTRY:-""}"
VERSION_TAG="${VERSION_TAG:-""}"
VERIFY_TLS="${VERIFY_TLS:-"true"}"
ALPINE_TAG="${ALPINE_TAG:-"latest"}"
ALIAS_TAGS="${ALIAS_TAGS:-""}"
DRY_RUN="${DRY_RUN:-"false"}"
PUBLISH="${PUBLISH:-"false"}"
PUBLISH="${PUBLISH,,}"
DRY_RUN="${DRY_RUN,,}"

# ANSI colors
RST='\033[0m'      # reset
C_G='\033[0;32m'   # color green
C_R='\033[0;31m'   # color red
C_Y='\033[0;33m'   # color yellow
C_Cy='\033[0;36m'  # color cyan

if [ -z "$IMAGE_NAME" ]; then
  echo -e "${C_R}Error: IMAGE_NAME variable is empty${RST}"
  exit 1
fi

if [[ -z "$REGISTRY" && "${PUBLISH}" == "true" ]]; then
  echo -e "${C_R}Error: REGISTRY variable is empty${RST}"
  exit 1
fi

if [ -z "$VERSION_TAG" ]; then
  echo -e "${C_R}Error: VERSION_TAG variable is empty${RST}"
  exit 1
fi

if [ "$VERIFY_TLS" != "true" ]; then
  echo -e "${C_Y}Warning TLS verification disabled.${RST}"
fi

if [ "${PUBLISH}" != "true" ]; then
  echo -e "${C_Cy}Info: Local build only. Images won't be published to the container registry.${RST}"
fi

BUILD_OPTIONS=("linux/arm64;aarch64-unknown-linux-musl" "linux/amd64;x86_64-unknown-linux-musl")
CREATED_IMAGES=();

for i in "${BUILD_OPTIONS[@]}";
do
  _PLATFORM=$(awk -F ';' '{print $1}' <<< "$i")
  _TOOLCHAIN=$(awk -F ';' '{print $2}' <<< "$i")
  _ARCH=$(awk -F '/' '{print $2}' <<< "${_PLATFORM}")
  _IMAGE="${IMAGE_NAME,,}:${VERSION_TAG,,}-${_ARCH,,}"

  echo -e "${C_Cy}Target details:${RST} ${C_Y}platform=${RST}${_PLATFORM};${C_Y}architecture=${RST}${_ARCH};${C_Y}rust_toolchain=${RST}${_TOOLCHAIN} ${RST}";
  echo -e "${C_Cy}Target alpine version: ${ALPINE_TAG} ${RST}";
  echo -e "${C_Cy}Output version: ${VERSION_TAG} ${RST}";
  echo -e "${C_Y}Building ${_IMAGE}${RST}";

  [[ "${DRY_RUN}" == "false" ]] && buildah build \
  --platform "${_PLATFORM}" \
  --layers \
  -f Dockerfile \
  --build-arg RUST_TOOLCHAIN="${_TOOLCHAIN}" \
  --build-arg ALPINE_TAG="${ALPINE_TAG}" \
  --build-arg VERSION_TAG="${VERSION_TAG}" \
  -t "${_IMAGE}"

  echo -e "${C_G}Container image '${_IMAGE}' built.${RST}";
  CREATED_IMAGES+=("${_IMAGE}");
done;

if [[ "$PUBLISH" == "true" ]];
then

  [[ "${DRY_RUN}" == "false" ]] && buildah login --tls-verify="${VERIFY_TLS}" "${REGISTRY}"

  for image in "${CREATED_IMAGES[@]}";
  do
    echo -e "${C_Y}Pushing '${image}' to '${REGISTRY,,}'.${RST}"
    [[ "${DRY_RUN}" == "false" ]] && buildah push --tls-verify="${VERIFY_TLS}" "${image}" "${REGISTRY,,}/${image}"
    echo -e "${C_G}Push container image '${image}' completed.${RST}"
  done;

  TAG_LIST=("${VERSION_TAG}" ${ALIAS_TAGS[@]});

  for tag in "${TAG_LIST[@]}";
  do
    if [ -n "$tag" ]; then
      _MANIFEST="${REGISTRY,,}/${IMAGE_NAME,,}:${tag,,}"

      echo -e "${C_Y}Creating manifest ${_MANIFEST}${RST}";
      [[ "${DRY_RUN}" == "false" ]] &&  buildah manifest create --tls-verify="${VERIFY_TLS}" "${_MANIFEST}"

      for image in "${CREATED_IMAGES[@]}";
      do
        echo -e "${C_Cy}Adding image '${REGISTRY}/${image}' to the manifest '${_MANIFEST}'${RST}'.";
        [[ "${DRY_RUN}" == "false" ]] && buildah manifest add --tls-verify="${VERIFY_TLS}" "${_MANIFEST}" "docker://${REGISTRY}/${image}";
      done;

      echo -e "${C_Y}Pushing manifest '${_MANIFEST}' to '${REGISTRY,,}'.${RST}"
      [[ "${DRY_RUN}" == "false" ]] && buildah manifest push --tls-verify="${VERIFY_TLS}" --all "${_MANIFEST}"
      echo -e "${C_G}Push manifest '${_MANIFEST}' completed.${RST}";
    fi
  done;
fi

echo -e "${C_G}Task Completed${RST}"