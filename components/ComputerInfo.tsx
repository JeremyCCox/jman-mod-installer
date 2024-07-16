import {type} from "@tauri-apps/api/os";
import {useQuery} from "react-query";

export default function ComputerInfo(){
    const osInfo = useQuery('osType',async () => {
        return (await type())
    })
    return(
        <>
            {osInfo.isLoading?
            <></>
            :
                osInfo.error?
                    <>Could not verify operating system!</>
                    :
                    osInfo.data
            }
        </>
    )
}