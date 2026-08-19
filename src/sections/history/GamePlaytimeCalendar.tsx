import { useEffect, useState } from 'react'
import { getSessions } from '../../api/sessions';
import { getDaysInCurrentYear, dateToDayIndex, scaleOpacity } from './calendarUtils';

const WEEK_DAYS_ARRAY = ["Mon", "Wed", "Fri"]
const MONTHS_ARRAY = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

const WeekDayLabels = () => (
  <div className='h-full flex flex-col items-center justify-between pt-10'>
    {WEEK_DAYS_ARRAY.map((day, idx) => (<span key={idx}>{day}</span>))}
  </div>
)

const MonthLabels = () => (
  <div className='flex flex-row w-full justify-between px-2'>
    {MONTHS_ARRAY.map((month, idx) => (<span key={idx}>{month}</span>))}
  </div>
)

const DayCell = ({ value, dayIndex }: { value: number; dayIndex: number }) => (
  <div
    className="w-5 h-5 rounded bg-primary opacity-(--cell-opacity)"
    style={{ '--cell-opacity': value } as React.CSSProperties}
    title={`Day ${dayIndex + 1}: ${value.toFixed(2)}`}
  />
)

const CalendarGrid = ({ dayChart }: { dayChart: number[] }) => (
  <div className="grid grid-rows-7 grid-flow-col gap-1">
    {dayChart.map((value, idx) => (
      <DayCell key={idx} value={value} dayIndex={idx} />
    ))}
  </div>
)

const GamePlaytimeCalendar = () => {
  const [dayChart, setDayChart] = useState<number[]>([])
  useEffect(() => {
    getSessions()
      .then((sessions) => {
        const chart = new Array(getDaysInCurrentYear()).fill(0.1);
        if (sessions.length === 0) {
          setDayChart(chart);
          return;
        }

        const durations = sessions.map((s) => s.duration);
        const minDuration = Math.min(...durations);
        const maxDuration = Math.max(...durations);

        for (const session of sessions) {
          const idx = dateToDayIndex(session.date);
          if (idx >= 0 && idx < chart.length) {
            chart[idx] = scaleOpacity(session.duration, minDuration, maxDuration);
          }
        }

        setDayChart(chart);
      })
      .catch(console.error);
  }, [])

  return (
    <div className="border-b-2 border-border pb-4">
      <div className='flex flex-row gap-3 items-center px-6 py-4 justify-center h-full '>
        <WeekDayLabels />
        <div className='flex flex-col gap-3'>
          <MonthLabels />
          <CalendarGrid dayChart={dayChart} />
        </div>
      </div>
    </div>
  )
}

export default GamePlaytimeCalendar
