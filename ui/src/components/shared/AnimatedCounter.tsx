import { useEffect, useRef, useState } from 'react';
import { animate } from 'framer-motion';

interface AnimatedCounterProps {
  value: number;
  duration?: number;
  format?: (n: number) => string;
}

export function AnimatedCounter({
  value,
  duration = 1,
  format = (n) => Math.round(n).toLocaleString(),
}: AnimatedCounterProps) {
  const [display, setDisplay] = useState(format(0));
  const prevValue = useRef(0);

  useEffect(() => {
    const controls = animate(prevValue.current, value, {
      duration,
      ease: 'easeOut',
      onUpdate: (v) => setDisplay(format(v)),
    });
    prevValue.current = value;
    return () => controls.stop();
  }, [value, duration, format]);

  return <span>{display}</span>;
}
