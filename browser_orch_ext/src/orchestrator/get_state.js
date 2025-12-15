async function getState() {
    const a = Array.from(document.querySelectorAll("*[data-r]"));
    return {
        elements: a.map((x) => {
            const r = x.getAttribute("data-r");
            const bounding = x.getBoundingClientRect();
            const isButton = x.tagName === "BUTTON" || x.type === "button" || x.type === "submit";
            return {
                attributes: Array.from(x.attributes).map((x) => [x.name, x.value]),
                // r value of the element
                R: r,
                // Name of the element (for buttons)
                name: isButton ? x.innerText : x.getAttribute("aria-label") ?? x.getAttribute("alt") ?? x.placeholder ?? "",
                // Various metadata about the element that is useful for the model
                metadata: {
                    tagName: x.tagName,
                    type: x.type,
                    hasValue: x.value != null && x.value !== "",
                    isChecked: x.checked ?? false,
                    isDisabled: x.disabled ?? false,
                    isRquired: x.required ?? false,
                    isReadOnly: x.readOnly ?? false,
                },
            };
        }),
        viewport: {
            x: window.scrollX,
            y: window.scrollY,
            width: window.innerWidth,
            height: window.innerHeight,
            scale: window.devicePixelRatio,
        },
    };
}

getState();