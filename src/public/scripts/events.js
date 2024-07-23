/**
 *
 * @param {*} event
 */
function parseEvent(src) {
  console.log("[event] received: ", src);
  const text = src.data.trim().split(",").join(" ");
  const name = src.event || "message";
  return `${name}: ${text}`;
}

/**
 * Create a new element with the given tag name, data and styles.
 */
function createRowElement({ tagName = "p", text = "", style } = {}) {
  const element = document.createElement(tagName);
  element.textContent = text;
  element.style = style;
  return element;
}

/**
 * Watch events from event source and display them in the target element.
 *
 * @param {*} config - configuration for event stream.
 *  @param {string} config.eventSource - event source url (i.e. http://localhost:3000/events)
 *  @param {string} config.targetElement - target element which contains event data.
 */
function watchEvents(
  config = {
    eventSource: "/events",
    targetElement: "event-stream",
    onErrorDisconnect: false,
  },
) {
  console.log("[event] watching events...");
  const eventSource = new EventSource(config.eventSource);
  const container = document.getElementById(config.targetElement);

  // incoming events
  eventSource.onmessage = (event) => {
    const data = parseEvent(event);
    const elem = createRowElement({ text: data });
    container.appendChild(elem);
  };

  // handle errors
  eventSource.onerror = (error) => {
    console.error("EventSource failed:", error);
    const reconnecting = config.onErrorDisconnect
      ? "terminated."
      : "re-connecting";

    const warn = createRowElement({
      text: `error: disconnected (${reconnecting})`,
      style: "color: red",
    });

    container.appendChild(warn);
    warn.scrollIntoView({ behavior: "smooth" });

    // close event source if error occurs and onErrorDisconnect is set
    if (config.onErrorDisconnect) eventSource.close();
  };
}

/**
 * Start watching events as soon as the page loads.
 */
watchEvents({
  targetElement: "event-stream",
  eventSource: "/events",
  onErrorDisconnect: false,
});
