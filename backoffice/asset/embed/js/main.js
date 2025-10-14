import htmx from './lib/htmx/htmx.esm.js'

async function stub() {
}

function formatToLocalTime() {
    stub().then(function () {
        let elements = document.getElementsByClassName("js-date-local");
        for (let element of elements) {
            let date = new Date(element.innerHTML);
            if (isNaN(date.getTime()) || date.toString() === "Invalid Date" || date.getTime() === 0) {
                return;
            }
            element.innerHTML = date.toLocaleString();
        }
    }).then(function () {
        let elements = document.getElementsByClassName("js-date-local");
        for (let element of elements) {
            element.classList.remove("js-date-local");
        }
    })
}

async function clearNavActive() {
    let elements = document.getElementsByClassName("nav-item");
    for (let element of elements) {
        element.classList.remove("nav-item-active");
    }
}

function addNavActive() {
    let tagUpdateElement = document.getElementById("tag-update");
    if (tagUpdateElement !== null) {
        clearNavActive().then(function () {
            if (tagUpdateElement.dataset.tag === undefined || tagUpdateElement.dataset.tag === "") {
                return;
            }
            let tagElement = document.getElementById(tagUpdateElement.dataset.tag);
            if (tagElement !== null) {
                tagElement.classList.add("nav-item-active");
            }
        }).then(function () {
            tagUpdateElement.remove();
        });
    }
}

export function start() {
    htmx.onLoad(function () {
        formatToLocalTime();
        addNavActive();
    });

    htmx.on("htmx:responseError", function (evt) {
        if (evt.detail.xhr.status === 422) {
            return;
        }

        let pre = document.createElement("pre");
        pre.classList.add("pre");
        pre.innerText = evt.detail.xhr.responseText;

        let div = document.createElement("div");
        div.innerHTML = "<h1>Error " + evt.detail.xhr.status + " " + evt.detail.xhr.statusText + "</h1><br>";
        div.appendChild(pre);

        htmx.swap("#main-content", div.outerHTML, {
            swapStyle: "innerHTML",
            swapDelay: 0,
            settleDelay: 0,
            transition: false,
            ignoreTitle: true,
            head: "<title>" + evt.detail.xhr.status + " " + evt.detail.xhr.statusText + "</title>",
            scroll: "top",
            show: "#main-content",
            focusScroll: true
        });
    });
}
