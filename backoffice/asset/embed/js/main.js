import htmx from './lib/htmx/htmx.esm.js'
import Alpine from './lib/alpine/alpine.esm.js'
import morph from './lib/alpine/plugin/morph.esm.js'

export function start() {
    Alpine.store('util', {
        /**
         * @param {HTMLElement} element
         */
        formatToLocalTime(element) {
            let date = new Date(element.innerHTML);
            if (isNaN(date.getTime()) || date.toString() === "Invalid Date" || date.getTime() === 0) {
                return;
            }
            element.innerHTML = date.toLocaleString();
        }
    });

    Alpine.store('nav', {
        /**
         * @returns {Promise<void>}
         */
        async clearActive() {
            let elements = document.getElementsByClassName("nav-item");
            for (let element of elements) {
                element.classList.remove("nav-item-active");
            }
        },
        /**
         * @param {string} tag
         * @returns {Promise<void>}
         */
        async updateActive(tag) {
            await this.clearActive();
            if (tag === "") {
                return;
            }
            let tagElement = document.getElementById(tag);
            if (tagElement !== null) {
                tagElement.classList.add("nav-item-active");
            }
        },
        /**
         * @param {HTMLElement} element
         * @returns {Promise<void>}
         */
        async updateActiveByElement(element) {
            if (element.dataset.tag) {
                await this.updateActive(element.dataset.tag);
            }
            element.remove();
        }
    });

    Alpine.store('csrf', {
        token: '',
        /** @param {string} token */
        updateToken(token) {
            if (this.token !== token) {
                this.token = token;
            }
        },
        /**
         * @param {HTMLElement} element
         * @param {boolean} remove
         */
        updateTokenByElement(element, remove = true) {
            if (element.dataset.csrf) {
                this.updateToken(element.dataset.csrf);
            }
            if (remove) {
                element.remove();
            }
        },
        /**
         * @param {string} url
         * @param {Object} options
         * @returns {Promise<Response>}
         */
        fetch(url, options = {}) {
            return fetch(url, {
                ...options,
                headers: {
                    ...options.headers,
                    'X-Csrf-Token': this.token
                }
            });
        }
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
            evt.detail.headers["X-Csrf-Token"] = Alpine.store('csrf').token;
        }
    });

    window.Alpine = Alpine;
    window.htmx = htmx;

    Alpine.plugin(morph);

    Alpine.start();
}
