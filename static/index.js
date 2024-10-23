async function updateCalendar(url) {

    let bottom_text = document.getElementsByClassName("bottom-message")[0];

    try {
        const res = await fetch(url)
            .then(res => res.json())
            .then(events => {
                // console.log(events);

                let dates = ["start", "end"];

                for (event in events) {
                    for (date in dates) {
                        events[event][dates[date]] = new Date(events[event][dates[date]] * 1000);
                    }
                }

                var init_date = null;
                if (events.length != 0) {
                    init_date = events[events.length - 1]["end"]; // initializes to the newest date in the data
                }

                var calendar = new tui.Calendar("#calendar", {

                    defaultView: "month",
                    template: {
                        time(event) {
                            const {start, end, title} = event;

                            return `<span style="color: white;">${formatTime(start)}~${formatTime(end)} ${title}</span>`;
                        },
                        allday(event) {
                            return `<span style="color: grey;">${event.title}</span>`;
                        },
                    },
                    calendars: events,
                });
                // calendar.render();

                bottom_text.innerText = "This report was automatically generated";
                bottom_text.className += " print-only";
            });
    } catch (e) {
        bottom_text.innerText = `Failed to get Git Repo! (Maybe try refreshing?)`;

        let error = document.createElement("b");
        error.innerText = `${e}`;
        error.setAttribute("style", "color: red;");

        bottom_text.insertAdjacentElement("beforeend", document.createElement("br"));
        bottom_text.insertAdjacentElement("beforeend", error);
        console.log(e);
    }
}
