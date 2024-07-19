function updateCalendar() {
    fetch("/api/data")
        .then(res => res.json())
        .then(events => {
            console.log(events);

            let dates = ["start", "end"];

            for (event in events) {
                for (date in dates) {
                    events[event][dates[date]] = new Date(events[event][dates[date]] * 1000);
                }
            }

            var calendarEl = document.getElementById("calendar");
            var calendar = new FullCalendar.Calendar(calendarEl, {
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

updateCalendar();
