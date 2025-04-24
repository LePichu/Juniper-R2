import { type Component, JSX, Suspense } from "solid-js";
import { A, useLocation } from "@solidjs/router";

const App: Component<{ children: JSX.Element }> = (
	props: { children: Element },
) => {
	const location = useLocation();

	return (
		<>
			<Suspense>{props.children}</Suspense>
		</>
	);
};

export default App;
