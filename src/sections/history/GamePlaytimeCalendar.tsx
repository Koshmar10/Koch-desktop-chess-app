
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react'

const WEEK_DAYS_ARRAY = ["Mon", "Wed", "Fri"]
const MONTHS_ARRAY = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

function getDaysInCurrentYear(): number {
  const year = new Date().getFullYear();
  const start = new Date(year, 0, 1);
  const end = new Date(year + 1, 0, 1);
  const diff = end.getTime() - start.getTime();
  return diff / (1000 * 60 * 60 * 24);
}

function generateRandomArray(length: number): number[] {
  return Array.from({ length }, () => Math.max(0.1, Math.random()));
}

const GamePlaytimeCalendar = () => {
  const [dayChart, setDayChart] = useState<number[]>([])
  useEffect(() => {
    invoke('get_sessions').then((message) => console.log(message));
    setDayChart(generateRandomArray(getDaysInCurrentYear()))
  }, [])

  return (
    <div className="border-b-2 border-border text-foreground pb-4">
      <div className='flex flex-row gap-4 items-center px-6 py-4 justify-center h-full '>
        <div className='h-full days flex flex-col gap-10 items-center justify-end'>{WEEK_DAYS_ARRAY.map((mon, idx) => (<span key={idx}>{mon}</span>))}</div>
        <div className='flex flex-col w-fit gap-2'>

          <div className='flex flex-row w-full justify-between pr-6'>{MONTHS_ARRAY.map((mon, idx) => (<span key={idx}>{mon}</span>))}</div>
          <div
            className="grid"
            style={{
              gridTemplateRows: 'repeat(7, 1fr)',
              gridAutoFlow: 'column',
              gap: '4px',
            }}
          >
            {dayChart.map((value, idx) => (
              <div
                key={idx}
                className="w-5 h-5 rounded bg-primary"
                style={{
                  opacity: value,
                }}
                title={`Day ${idx + 1}: ${value.toFixed(2)}`}
              />
            ))}
          </div>
        </div>
      </div>

    </div>
  )
}

export default GamePlaytimeCalendar