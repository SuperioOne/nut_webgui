/**
 * @param {ShadowRoot} root
 * @returns {void}
 */
export function link_host_styles(root) {
  for (const sheet of document.styleSheets) {
    if (!sheet.href) continue;

    const link = document.createElement("link");

    link.rel = "stylesheet";
    link.href = sheet.href;
    root.prepend(link);
  }
}
