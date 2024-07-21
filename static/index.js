function updateCalendar(url) {

    fetch(url)
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

            var calendarEl = document.getElementById("calendar");
            var calendar = new FullCalendar.Calendar(calendarEl, {
                initialDate: init_date,
                initialView: "dayGridMonth",
                headerToolbar: {
                    "left": "prev,next today",
                    "center": "title",
                    "right": "dayGridMonth,timeGridWeek,timeGridDay,listWeek"
                },
                events: events
            });
            calendar.render();
        });

}
