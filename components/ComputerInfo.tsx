import {type} from "@tauri-apps/api/os";
import {useQuery} from "react-query";
import {invoke} from "@tauri-apps/api";

export default function ComputerInfo(){
    const osInfo = useQuery('osType',async () => {
        return (await type())
    })

    const writeClick=async ()=>{
        await invoke("write_installer_config",{installerConfig:{defaultGameDir:"test"}})
    }
    const readClick=async ()=>{
        console.log(await invoke("read_installer_config"))
    }

    return(
        <>
            {osInfo.isLoading?
            <></>
            :
                osInfo.error?
                    <>Could not verify operating system!</>
                    :
                    <div className={'w-full h-8 flex justify-evenly'}>
                        {osInfo.data}
                        <input type={'button'} onClick={readClick} value={'read'}/>
                        <input type={'button'} onClick={writeClick} value={'write'}/>

                    </div>
            }
        </>
    )
}