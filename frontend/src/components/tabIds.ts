/** The DOM id of the tab `tabId` of the tab bar that controls `panelId`. */
export function tabElementId(panelId: string, tabId: string): string {
  return `${panelId}-tab-${tabId}`;
}
