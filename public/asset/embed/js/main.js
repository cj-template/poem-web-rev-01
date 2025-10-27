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

let token = "";
let rotateToken = true;

function refreshCsrfToken() {
    if (rotateToken) {
        rotateToken = false;
        fetch("/csrf/token").then(function (response) {
            return response.json();
        }).then(function (data) {
            token = data.token;
            updateTokenInput(data.token);
            return data.token;
        }).catch(function (error) {
            console.error(error);
        });
    } else {
        fetchTokenFromInput();
    }
}

function updateTokenInput(newToken) {
    let tokenInputs = document.querySelectorAll("input[name='csrf_token']");
    for (let tokenInput of tokenInputs) {
        tokenInput.value = newToken;
    }
}

function fetchTokenFromInput() {
    let tokenInputs = document.querySelectorAll("input[name='csrf_token']");
    let tokenInput = tokenInputs[0];
    if (tokenInput !== undefined) {
        token = tokenInput.value;
    }
}

function getCsrfToken() {
    rotateToken = true;
    return token;
}

export function start() {
    htmx.onLoad(function () {
        formatToLocalTime();
        addNavActive();
        refreshCsrfToken();
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

    document.body.addEventListener("htmx:configRequest", function (evt) {
        if (evt.detail.verb !== "get" && evt.detail.verb !== "head") {
            evt.detail.headers["X-Csrf-Token"] = getCsrfToken();
        }
    });

    window.csrfToken = getCsrfToken;
}
