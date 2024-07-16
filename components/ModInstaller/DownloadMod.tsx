import {useQuery} from "react-query";

export default function DownloadMod(){

    const modDownload = useQuery("modDownload",()=>{
        fetch("https://www.curseforge.com/minecraft/mc-mods/dynamictreesplus/download/5344010").then(res=>{
            console.log(res)
            return("res done")
        })
    })
    return(
        <>
            {modDownload.isLoading&&<>Loading</>}
        </>
    )
}