export function CompactRecordingIndicator() {
  return (
    <div className="flex items-center gap-[2px] h-4" title="Recording">
      {[0, 1, 2, 3].map((i) => (
        <div
          key={i}
          className="w-[3px] rounded-full bg-destructive"
          style={{
            animation: `compact-wave 0.8s ease-in-out ${i * 0.15}s infinite alternate`,
          }}
        />
      ))}
    </div>
  );
}
