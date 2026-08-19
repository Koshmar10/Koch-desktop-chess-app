import { useEffect, useState } from "react";
import { getSessions } from "../../api/sessions";
import { Tooltip } from "../../components/Tooltip";
import {
  getDaysInCurrentYear,
  dateToDayIndex,
  scaleOpacity,
  formatDayLabel,
  formatDuration,
} from "./calendarUtils";

const WEEK_DAYS_ARRAY = ["Mon", "Wed", "Fri"];
const MONTHS_ARRAY = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

interface DayData {
  opacity: number;
  tooltip: string | null;
}

const EMPTY_DAY: DayData = { opacity: 0.1, tooltip: null };

const WeekDayLabels = () => (
  <div className="h-full flex flex-col items-center justify-between pt-10">
    {WEEK_DAYS_ARRAY.map((day, idx) => (
      <span key={idx}>{day}</span>
    ))}
  </div>
);

const MonthLabels = () => (
  <div className="flex flex-row w-full justify-between px-2">
    {MONTHS_ARRAY.map((month, idx) => (
      <span key={idx}>{month}</span>
    ))}
  </div>
);

const DayCell = ({ opacity, tooltip }: DayData) => (
  <Tooltip label={tooltip}>
    <div className="w-5 h-5 rounded bg-primary" style={{ opacity }} />
  </Tooltip>
);

const CalendarGrid = ({ dayChart }: { dayChart: DayData[] }) => (
  <div className="grid grid-rows-7 grid-flow-col gap-1">
    {dayChart.map((day, idx) => (
      <DayCell key={idx} opacity={day.opacity} tooltip={day.tooltip} />
    ))}
  </div>
);

const GamePlaytimeCalendar = () => {
  const [dayChart, setDayChart] = useState<DayData[]>([]);
  useEffect(() => {
    getSessions()
      .then((sessions) => {
        const chart: DayData[] = new Array(getDaysInCurrentYear()).fill(
          EMPTY_DAY,
        );

        for (const session of sessions) {
          const idx = dateToDayIndex(session.date);
          if (idx >= 0 && idx < chart.length) {
            chart[idx] = {
              opacity: scaleOpacity(session.duration),
              tooltip: `${formatDayLabel(session.date)}: ${formatDuration(session.duration)}`,
            };
          }
        }

        setDayChart(chart);
      })
      .catch(console.error);
  }, []);

  return (
    <div className="border-b-2 border-border pb-4">
      <div className="flex flex-row gap-3 items-center px-6 py-4 justify-center h-full ">
        <WeekDayLabels />
        <div className="flex flex-col gap-3">
          <MonthLabels />
          <CalendarGrid dayChart={dayChart} />
        </div>
      </div>
    </div>
  );
};

export default GamePlaytimeCalendar;
