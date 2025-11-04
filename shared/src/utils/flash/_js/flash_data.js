({
    show: true,
    start() {
        setTimeout(() => {
            this.show = false;
            setTimeout(() => {
                $el.remove();
            }, 1000);
        }, 5000);
    }
})
