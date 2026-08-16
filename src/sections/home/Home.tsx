import HomeCard from "./HomeCard";
import { STAT_CARDS } from "./mock";

const GreetMessage = () => (
  <>
    <h1 className="text-3xl mt-6 mb-6 text-foreground">Welcome back, player</h1>
    <div className="text-md text-center text-foreground/60 italic self-center">
      With great power comes great responsability
    </div>
  </>
);

const StatCardsSection = () => (
  <div className="grid grid-cols-4 grid-rows-4 gap-4 w-[65%] h-[50rem] pt-20 px-4">
    {STAT_CARDS.map((card) => (
      <HomeCard key={card.label} {...card} />
    ))}
  </div>
);

const Home = () => {
  return (
    <div className="flex flex-col justify-start items-center w-full">
      <GreetMessage />
      <StatCardsSection />
    </div>
  );
};

export default Home;
