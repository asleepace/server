/**
 * Parse the incoming event data and return a formatted string.
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
 * Check if the container is near the bottom. (50px)
 */
function isNearBottom(container, threshold = 50) {
    return (
        container.scrollHeight - container.scrollTop - container.clientHeight <
        threshold
    );
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

    // append child to container and scroll to bottom (if close)
    function insertChildAndScroll(elem) {
        container.appendChild(elem);
        if (isNearBottom(container)) {
            elem.scrollIntoView({ behavior: "smooth" });
        }
    }

    eventSource.onopen = (event) => {
        console.log("[event] connected!", event);
        const ascii = [
            "      ____                      _          _____                     ",
            "     / ___|___  _ __ ___  _ __ | | ___    |  ___|__  _ __ _ __  _   _ ",
            "    | |   / _ \\| '_ ` _ \\| '_ \\| |/ _ \\   | |_ / _ \\| '__| '_ \\| | | |",
            "    | |__| (_) | | | | | | |_) | |  __/   |  _| (_) | |  | | | | |_| |",
            "     \\____\\___/|_| |_| |_| .__/|_|\\___|   |_|  \\___/|_|  |_| |_|\\__, |",
            "                         |_|                                      |___/ ",
            "",
            " :: welcome to ConsoleDump | streaming session ::",
            " :: tip: send POSTs to this URL to see them live ::",
        ].join("\n");
        const elem = createRowElement({ text: ascii, style: "color:#7efc7a" });
        insertChildAndScroll(elem);
    };

    // listen to base64 events
    eventSource.addEventListener("base64", (event) => {
        console.log("[event] received base64 event: ", event.data);
        const base64 = atob(event.data);
        const elem = createRowElement({ text: base64 });
        insertChildAndScroll(elem);
    });

    // incoming events
    eventSource.onmessage = (event) => {
        const data = parseEvent(event);
        const elem = createRowElement({ text: data });
        insertChildAndScroll(elem);
    };

    function getEventSourceStatus() {
        switch (eventSource.readyState) {
            case EventSource.CONNECTING:
                return "CONNECTING";
            case EventSource.OPEN:
                return "OPEN";
            case EventSource.CLOSED:
                return "CLOSED";
        }
    }

    // handle errors
    eventSource.onerror = (error) => {
        console.error("EventSource failed:", error);
        if (config.onErrorDisconnect) eventSource.close();
        const status = getEventSourceStatus();
        const warn = createRowElement({
            text: `error: disconnected (${status})`,
            style: "color: red",
        });

        insertChildAndScroll(warn);
    };
}



document.addEventListener('DOMContentLoaded', () => {
    console.log('[session.js] initializing ....')

    const [_, sessionKey] = window.location.pathname.split('s/')
    console.log('[session.js] sessionId:', sessionKey)

    /**
     * Start watching events as soon as the page loads.
     */
    watchEvents({
        targetElement: "event-stream",
        eventSource: `/events?s=${sessionKey}`,
        onErrorDisconnect: false,
    });
})