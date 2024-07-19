function updateCalendar() {
    fetch("/api/data")
        .then(res => res.json())
        .then(events => {
            // console.log(events);

            let dates = ["start", "end"];

            for (event in events) {
                for (date in dates) {
                    events[event][dates[date]] = new Date(events[event][dates[date]] * 1000);
                }
            }

            let init_date = events[events.length - 1]["end"]; // initializes to the newest date in the data

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

    fetch("/api/repo-name")
        .then(res => res.text())
        .then(repo_title => {
            console.log("Repo Title: " + repo_title);
            document.getElementById("title").innerText = repo_title;
        });

    fetch("/api/repo-url")
        .then(res => res.text())
        .then(repo_url => {
            console.log("Repo URL: " + repo_url);
            if (repo_url) {
                document.getElementById("title").setAttribute("href", repo_url);
            }
        });
}

document.addEventListener("DOMContentLoaded", () => {
    updateCalendar();
});
