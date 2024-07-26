import {QueryClient, QueryClientProvider} from "react-query";
import ComputerInfo from "../ComputerInfo.tsx";
import MinecraftInfo from "../MinecraftInfo.tsx";

export default function InstallerBase(){
    const queryClient = new QueryClient()

    return(
        <QueryClientProvider client={queryClient}>
            {/*<input type={'button'} value={'Click me'} onClick={logSftp}/>*/}
            <ComputerInfo/>
            <MinecraftInfo/>
        </QueryClientProvider>
    )
}