// Open-source and privacy-friendly tracking script from
// https://cdn.counter.dev/script.js. Aggregates statistics such as total page
// views, screen sizes and country of origin. Kept in the repository because I
// don't want to dynamically load javascript.
//
// I'm not usually a fan of collecting data, but this provides useful feedback
// for me to see how many people are using the tool. If you use an adblocker,
// such as uBlock Origin, it will likely block this tracking.
(function() {
    if (sessionStorage.getItem('doNotTrack') || localStorage.getItem('doNotTrack')) {
        return
    }
    var id = document.currentScript.getAttribute("data-id");
    var server = document.currentScript.getAttribute("data-server") || "https://t.counter.dev";
    if (!sessionStorage.getItem("_swa") && !document.referrer.startsWith(location.protocol + "//" + location.host)) {
        setTimeout(function() {
            sessionStorage.setItem("_swa", "1");
            fetch(server +
                "/track?" +
                new URLSearchParams({
                    referrer: document.referrer,
                    screen: screen.width + "x" + screen.height,
                    id: id,
                    utcoffset: 0,
                }));
        }, 4500);
    }
    navigator.sendBeacon(server + "/trackpage", new URLSearchParams({
        id: id,
        page: window.location.pathname,
    }));
})();
